//! This is the underlying API for the manager. More direct control over API calls are contained
//! within this modoule.

pub mod response;

mod builder;
mod response_wrappers;
mod helpers;

use response::*;
use response_wrappers::*;

pub use builder::SteamTradeOfferAPIBuilder;

use crate::SteamID;
use crate::helpers::get_default_middleware;
use crate::time::ServerTime;
use crate::types::*;
use crate::response::*;
use crate::enums::{Language, GetUserDetailsMethod};
use crate::static_functions::get_inventory;
use crate::serialize;
use crate::helpers::{parses_response, generate_sessionid, get_sessionid_and_steamid_from_cookies};
use crate::error::{Error, ParameterError, MissingClassInfoError};
use crate::classinfo_cache::{ClassInfoCache, helpers as classinfo_cache_helpers};
use crate::request::{GetInventoryOptions, NewTradeOffer, NewTradeOfferItem, GetTradeHistoryOptions};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use reqwest::cookie::Jar;
use reqwest::header::REFERER;
use lazy_regex::{regex_captures, regex_is_match};
use url::Url;

/// The underlying API for interacting with Steam trade offers.
#[derive(Debug, Clone)]
pub struct SteamTradeOfferAPI {
    /// The API key.
    pub api_key: Option<String>,
    /// The language for descriptions.
    pub language: Language,
    /// The client for making requests.
    client: Client,
    /// The cookies to make requests with. Since the requests are made with the provided client, 
    /// the cookies should be the same as what the client uses.
    cookies: Arc<Jar>,
    /// The session ID.
    sessionid: Arc<RwLock<Option<String>>>,
    /// The cache for setting and getting [`ClassInfo`] data.
    classinfo_cache: ClassInfoCache,
    /// The directory to store [`ClassInfo`] data.
    data_directory: PathBuf,
}

impl SteamTradeOfferAPI {
    /// Hostname for requests.
    const HOSTNAME: &'static str = "steamcommunity.com";
    /// Hostname for API requests.
    const API_HOSTNAME: &'static str = "api.steampowered.com";
    
    /// Builder for constructing a [`SteamTradeOfferAPI`].
    pub fn builder() -> SteamTradeOfferAPIBuilder {
        SteamTradeOfferAPIBuilder::new()
    }
    
    fn get_uri(
        &self,
        pathname: &str,
    ) -> String {
        format!("https://{}{pathname}", Self::HOSTNAME)
    }
    
    fn get_api_url(
        &self,
        interface: &str,
        method: &str,
        version: usize,
    ) -> String {
        format!("https://{}/{interface}/{method}/v{version}", Self::API_HOSTNAME)
    }
    
    /// Sets cookies.
    /// 
    /// Some features will only work if cookies are set, such as sending or responding to trade 
    /// offers. Make sure your cookies are set before calling these methods.
    pub fn set_cookies(
        &self,
        cookies: &[String],
    ) {
        let (sessionid, _steamid) = get_sessionid_and_steamid_from_cookies(cookies);
        let mut cookies = cookies.to_owned();
        let sessionid = if let Some(sessionid) = sessionid {
            sessionid
        } else {
            // the cookies don't contain a sessionid
            let sessionid = generate_sessionid();
            
            cookies.push(format!("sessionid={sessionid}"));
            sessionid
        };
        let url = format!("https://{}", Self::HOSTNAME).parse::<Url>()
            .unwrap_or_else(|_| panic!("URL could not be parsed from {}", Self::HOSTNAME));
        
        *self.sessionid.write().unwrap() = Some(sessionid);
        
        for cookie_str in &cookies {
            self.cookies.add_cookie_str(cookie_str, &url);
        }
    }
    
    /// Sends an offer.
    pub async fn send_offer(
        &self,
        offer: &NewTradeOffer,
        counter_tradeofferid: Option<TradeOfferId>,
    ) -> Result<SentOffer, Error> {
        #[derive(Serialize)]
        struct OfferFormUser<'b> {
            assets: &'b Vec<NewTradeOfferItem>,
            currency: Vec<Currency>,
            ready: bool,
        }

        #[derive(Serialize)]
        struct OfferForm<'b> {
            newversion: bool,
            version: u32,
            me: OfferFormUser<'b>,
            them: OfferFormUser<'b>,
        }

        #[derive(Serialize)]
        struct TradeOfferCreateParams<'b> {
            #[serde(skip_serializing_if = "Option::is_none")]
            trade_offer_access_token: &'b Option<String>,
        }

        #[derive(Serialize)]
        struct SendOfferParams<'b> {
            sessionid: String,
            serverid: u32,
            json_tradeoffer: String,
            tradeoffermessage: &'b Option<String>,
            captcha: &'static str,
            trade_offer_create_params: String,
            tradeofferid_countered: &'b Option<u64>,
            #[serde(serialize_with = "serialize::steamid_as_string")]
            partner: &'b SteamID,
        }
        
        #[derive(Serialize)]
        struct RefererParams<'b> {
            partner: u32,
            token: &'b Option<String>,
        }
        
        let num_items = offer.items_to_give.len() + offer.items_to_receive.len();
        
        if num_items == 0 {
            return Err(Error::Parameter(ParameterError::EmptyOffer));
        }
        
        let sessionid = self.sessionid.read().unwrap().clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = {
            let pathname: String = match &counter_tradeofferid {
                Some(id) => id.to_string(),
                None => String::from("new"),
            };
            let qs_params = serde_qs::to_string(&RefererParams {
                partner: offer.partner.account_id(),
                token: &offer.token,
            }).map_err(ParameterError::SerdeQS)?;
            
            self.get_uri(&format!("/tradeoffer/{pathname}?{qs_params}"))
        };
        let params = {
            let json_tradeoffer = serde_json::to_string(&OfferForm {
                newversion: true,
                // this is hopefully safe enough
                version: num_items as u32 + 1,
                me: OfferFormUser {
                    assets: &offer.items_to_give,
                    currency: Vec::new(),
                    ready: false,
                },
                them: OfferFormUser {
                    assets: &offer.items_to_receive,
                    currency: Vec::new(),
                    ready: false,
                },
            })?;
            let trade_offer_create_params = serde_json::to_string(&TradeOfferCreateParams {
                trade_offer_access_token: &offer.token,
            })?;
            
            SendOfferParams {
                sessionid,
                serverid: 1,
                captcha: "",
                tradeoffermessage: &offer.message,
                partner: &offer.partner,
                json_tradeoffer,
                trade_offer_create_params,
                tradeofferid_countered: &counter_tradeofferid,
            }
        };
        let uri = self.get_uri("/tradeoffer/new/send");
        let response = self.client.post(&uri)
            .header(REFERER, referer)
            .form(&params)
            .send()
            .await?;
        let body: SentOffer = parses_response(response).await?;
        
        Ok(body)
    }
    
    /// Gets the trade receipt (new items) upon completion of a trade.
    pub async fn get_receipt(
        &self,
        trade_id: &TradeId,
    ) -> Result<Vec<Asset>, Error> {
        let uri = self.get_uri(&format!("/trade/{trade_id}/receipt"));
        let response = self.client.get(&uri)
            .send()
            .await?;
        let body = response.text().await?;
        
        if let Some((_, message)) = regex_captures!(r#"<div id="error_msg">\s*([^<]+)\s*</div>"#, &body) {
           Err(Error::UnexpectedResponse(message.trim().into()))
        } else if let Some((_, script)) = regex_captures!(r#"(var oItem;[\s\S]*)</script>"#, &body) {
            let raw_assets = helpers::parse_receipt_script(script)?;
            let classes = raw_assets
                .iter()
                .map(|item| (item.appid, item.classid, item.instanceid))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let map = self.get_asset_classinfos(&classes).await?;
            let assets = raw_assets
                .into_iter()
                .map(|asset| helpers::from_raw_receipt_asset(asset, &map))
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(assets)
        } else if regex_is_match!(r#"\{"success": ?false\}"#, &body) {
            Err(Error::NotLoggedIn)
        } else {
            Err(Error::MalformedResponse)
        }
    }
    
    /// Gets a chunk of [`ClassInfo`] data.
    async fn get_app_asset_classinfos_chunk(
        &self,
        appid: AppId,
        classes: &[ClassInfoAppClass],
    ) -> Result<ClassInfoMap, Error> {
        let query = {
            let key = self.api_key.as_ref()
                .ok_or(ParameterError::MissingApiKey)?;
            let mut query = vec![
                ("key".into(), key.into()),
                ("appid".into(), appid.to_string()),
                ("language".into(), self.language.web_api_language_code().to_string()),
                ("class_count".into(), classes.len().to_string()),
            ];
            
            for (i, (classid, instanceid)) in classes.iter().enumerate() {
                query.push((format!("classid{i}"), classid.to_string()));
                
                if let Some(instanceid) = instanceid {
                    query.push((format!("instanceid{i}"), instanceid.to_string()));
                }
            }
            
            query
        };
        let uri = self.get_api_url("ISteamEconomy", "GetAssetClassInfo", 1);
        let response = self.client.get(&uri)
            .query(&query)
            .send()
            .await?;
        let body: GetAssetClassInfoResponse = parses_response(response).await?;
        let classinfos = body.result;
        
        classinfo_cache_helpers::save_classinfos(
            appid,
            &classinfos,
            &self.data_directory,
        ).await;
        
        let classinfos = classinfos
            .into_iter()
            // Sometimes Steam returns empty classinfo data.
            // We just ignore them until they are successfully fetched.
            .filter_map(|((classid, instanceid), classinfo_string)| {
                serde_json::from_str::<ClassInfo>(&classinfo_string)
                    // ignore classinfos that failed parsed
                    .ok()
                    .map(|classinfo| (
                        (appid, classid, instanceid),
                        Arc::new(classinfo),
                    ))
            })
            .collect::<HashMap<_, _>>();
        
        self.classinfo_cache.insert_map(&classinfos);

        Ok(classinfos)
    }
    
    /// Gets [`ClassInfo`] data for appid.
    async fn get_app_asset_classinfos(
        &self,
        appid: AppId,
        classes: Vec<ClassInfoAppClass>,
    ) -> Result<Vec<ClassInfoMap>, Error> {
        let chunk_size = 100;
        let chunks = classes.chunks(chunk_size);
        let mut maps = Vec::with_capacity(chunks.len());
        
        for chunk in chunks {
            maps.push(self.get_app_asset_classinfos_chunk(appid, chunk).await?);
        }
        
        Ok(maps)
    }
    
    /// Gets [`ClassInfo`] data for the given classes.
    pub async fn get_asset_classinfos(
        &self,
        classes: &Vec<ClassInfoClass>,
    ) -> Result<ClassInfoMap, Error> {
        if classes.is_empty() {
            return Ok(Default::default());
        }
        
        let mut apps: HashMap<AppId, Vec<ClassInfoAppClass>> = HashMap::new();
        // Check memory for caches.
        let (
            mut map,
            misses,
        ) = self.classinfo_cache.get_map(&classes);
        let mut needed = HashSet::from_iter(misses);
        
        if !needed.is_empty() {
            // check filesystem for caches
            let results = classinfo_cache_helpers::load_classinfos(
                &needed,
                &self.data_directory,
            ).await
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            
            if !results.is_empty() {
                let mut inserts = HashMap::new();
                
                for (class, classinfo) in results {
                    let classinfo = Arc::new(classinfo);
                    
                    needed.remove(&class);
                    inserts.insert(class, Arc::clone(&classinfo));
                }
                
                self.classinfo_cache.insert_map(&inserts);
                map.extend(inserts);
            }
        }
        
        for (appid, classid, instanceid) in needed {
            match apps.get_mut(appid) {
                Some(classes) => {
                    classes.push((*classid, *instanceid));
                },
                None => {
                    apps.insert(*appid, vec![(*classid, *instanceid)]);
                },
            }
        }
        
        for (appid, classes) in apps {
            for maps in self.get_app_asset_classinfos(appid, classes).await? {
                map.extend(maps);
            }
        }
        
        Ok(map)
    }
    
    /// Gets trade offer data before any descriptions are added. The 2nd part of the tuple are the 
    /// descriptions from the response if `get_descriptions` was set. These can be combined with 
    /// the offers using the `map_raw_trade_offers_with_descriptions` method.
    pub async fn get_raw_trade_offers(
        &self,
        active_only: bool,
        historical_only: bool,
        get_sent_offers: bool,
        get_received_offers: bool,
        get_descriptions: bool,
        historical_cutoff: Option<ServerTime>,
    ) -> Result<(Vec<response::RawTradeOffer>, Option<ClassInfoMap>), Error> {
        #[derive(Serialize)]
        struct Form<'a> {
            key: &'a str,
            language: &'a str,
            active_only: bool,
            historical_only: bool,
            get_sent_offers: bool,
            get_received_offers: bool,
            get_descriptions: bool,
            time_historical_cutoff: Option<u64>,
            cursor: Option<u32>,
        }
        
        let uri = self.get_api_url("IEconService", "GetTradeOffers", 1);
        let key = self.api_key.as_ref()
            .ok_or(ParameterError::MissingApiKey)?;
        let mut cursor = None;
        let time_historical_cutoff = historical_cutoff
            .map(|cutoff| cutoff.timestamp() as u64);
        let mut offers = Vec::new();
        let mut descriptions = Vec::new();
        
        loop {
            let response = self.client.get(&uri)
                .query(&Form {
                    key,
                    language: self.language.web_api_language_code(),
                    active_only,
                    historical_only,
                    get_sent_offers,
                    get_received_offers,
                    get_descriptions,
                    time_historical_cutoff,
                    cursor,
                })
                .send()
                .await?;
            let body: GetTradeOffersResponse = parses_response(response).await?;
            let next_cursor = body.response.next_cursor;
            let mut response = body.response;
            let mut response_offers = response.trade_offers_received;
            
            if let Some(response_descriptions) = response.descriptions {
                descriptions.push(response_descriptions);
            }
            
            response_offers.append(&mut response.trade_offers_sent);
            
            if let Some(historical_cutoff) = historical_cutoff {
                // Is there an offer older than the cutoff?
                let has_older = response_offers
                    .iter()
                    .any(|offer| offer.time_created < historical_cutoff);
                
                // we don't need to go any further...
                if has_older {
                    // add the offers, filtering out older offers
                    offers.append(&mut response_offers);
                    break;
                }
            }
            
            offers.append(&mut response_offers);
            
            if next_cursor > Some(0) {
                cursor = next_cursor;
            } else {
                break;
            }
        }
        
        let descriptions = if !descriptions.is_empty() {
            let combined = descriptions
                .into_iter()
                .flatten()
                .collect::<HashMap<_, _>>();
            
            Some(combined)
        } else {
            None
        };
        
        Ok((offers, descriptions))
    }
    
    /// Maps trade offer data with descriptions from the cache and API. Ignores offers with 
    /// missing descriptions.
    pub async fn map_raw_trade_offers(
        &self,
        offers: Vec<RawTradeOffer>,
    ) -> Result<Vec<TradeOffer>, Error> {
        let classes = offers
            .iter()
            .flat_map(|offer| {
                offer.items_to_give
                    .iter()
                    .chain(offer.items_to_receive.iter())
                    .map(|item| (item.appid, item.classid, item.instanceid))
            })
            // make unique
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let map = self.get_asset_classinfos(&classes).await?;
        let offers = self.map_raw_trade_offers_with_descriptions(offers, map);
        
        Ok(offers)
    }
    
    /// Maps trade offer data with given descriptions. Ignores offers with missing descriptions.
    pub fn map_raw_trade_offers_with_descriptions(
        &self,
        offers: Vec<RawTradeOffer>,
        map: ClassInfoMap,
    ) -> Vec<TradeOffer> {
        offers
            .into_iter()
            // ignore offers where the classinfo cannot be obtained
            // attempts to load the missing classinfos will continue
            // but it will not cause the whole poll to fail
            .filter_map(|offer| offer.try_combine_classinfos(&map).ok())
            .collect()
    }
    
    /// Gets trade offers.
    pub async fn get_trade_offers(
        &self,
        active_only: bool,
        historical_only: bool,
        get_sent_offers: bool,
        get_received_offers: bool,
        get_descriptions: bool,
        historical_cutoff: Option<ServerTime>,
    ) -> Result<Vec<TradeOffer>, Error> {
        let (raw_offers, _descriptions) = self.get_raw_trade_offers(
            active_only,
            historical_only,
            get_sent_offers,
            get_received_offers,
            get_descriptions,
            historical_cutoff,
        ).await?;
        let offers = self.map_raw_trade_offers(raw_offers).await?;
        
        Ok(offers)
    }
    
    /// Gets a trade offer.
    pub async fn get_trade_offer(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<response::RawTradeOffer, Error> {
        #[derive(Serialize)]
        struct Form<'a> {
            key: &'a str,
            tradeofferid: TradeOfferId,
        }
        
        #[derive(Deserialize, Debug)]
        struct Body {
            offer: response::RawTradeOffer,
        }
        
        #[derive(Deserialize, Debug)]
        struct Response {
            response: Body,
        }
        
        let uri = self.get_api_url("IEconService", "GetTradeOffer", 1);
        let key = self.api_key.as_ref()
            .ok_or(ParameterError::MissingApiKey)?;
        let response = self.client.get(&uri)
            .query(&Form {
                key,
                tradeofferid,
            })
            .send()
            .await?;
        let body: Response = parses_response(response).await?;
        
        Ok(body.response.offer)
    }
    
    /// Gets trade history. The second part of the returned tuple is whether more trades can be 
    /// fetched.
    pub async fn get_trade_history(
        &self,
        options: &GetTradeHistoryOptions,
    ) -> Result<Trades, Error> {
        let body = self.get_trade_history_request(
            options.max_trades,
            options.start_after_time,
            options.start_after_tradeid,
            options.navigating_back,
            true,
            options.include_failed,
            true,
        ).await?;
        
        if let Some(descriptions) = body.descriptions {
            let trades = body.trades
                .into_iter()
                .map(|trade| trade.try_combine_classinfos(&descriptions))
                .collect::<Result<_, _>>()?;
                
            Ok(Trades {
                trades,
                more: body.more,
                // Should always be present since include_total was passed.
                total_trades: body.total_trades.unwrap_or_default(),
            })
        } else {
            Err(Error::UnexpectedResponse("No descriptions in response body.".into()))
        }
    }
    
    /// Gets trade history without descriptions. The second part of the returned tuple is whether 
    /// more trades can be fetched.
    pub async fn get_trade_history_without_descriptions(
        &self,
        options: &GetTradeHistoryOptions,
    ) -> Result<RawTrades, Error> {
        let body = self.get_trade_history_request(
            options.max_trades,
            options.start_after_time,
            options.start_after_tradeid,
            options.navigating_back,
            false,
            options.include_failed,
            true,
        ).await?;
        
        Ok(RawTrades {
            trades: body.trades,
            more: body.more,
            // Should always be present since include_total was passed.
            total_trades: body.total_trades.unwrap_or_default(),
        })
    }
    
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
    async fn get_trade_history_request(
        &self,
        max_trades: u32,
        start_after_time: Option<u32>,
        start_after_tradeid: Option<TradeId>,
        navigating_back: bool,
        get_descriptions: bool,
        include_failed: bool,
        include_total: bool,
    ) -> Result<GetTradeHistoryResponseBody, Error> {
        #[derive(Serialize)]
        struct Form<'a> {
            key: &'a str,
            max_trades: u32,
            start_after_time: Option<u32>,
            start_after_tradeid: Option<TradeId>,
            navigating_back: bool,
            get_descriptions: bool,
            include_failed: bool,
            include_total: bool,
        }
        
        let uri = self.get_api_url("IEconService", "GetTradeHistory", 1);
        let key = self.api_key.as_ref()
            .ok_or(ParameterError::MissingApiKey)?;
        let response = self.client.get(&uri)
            .query(&Form {
                key,
                max_trades,
                start_after_time,
                start_after_tradeid,
                navigating_back,
                get_descriptions,
                include_failed,
                include_total,
            })
            .send()
            .await?;
        let body: GetTradeHistoryResponse = parses_response(response).await?;
        
        Ok(body.response)
    }
    
    /// Gets escrow details for a user. The `method` for obtaining details can be a `tradeofferid` 
    /// or `access_token` or neither.
    pub async fn get_user_details<T>(
        &self,
        partner: SteamID,
        method: T,
    ) -> Result<UserDetails, Error> 
        where T: Into<GetUserDetailsMethod>,
    {
        #[derive(Serialize)]
        struct Params<'b> {
            partner: u32,
            token: Option<&'b str>,
        }
        
        let uri = {
            let method = method.into();
            let pathname = method.pathname();
            let qs_params = serde_qs::to_string(&Params {
                partner: partner.account_id(),
                token: method.token(),
            }).map_err(ParameterError::SerdeQS)?;
            
            self.get_uri(&format!("/tradeoffer/{pathname}?{qs_params}"))
        };
        let response = self.client.get(&uri)
            .send()
            .await?;
        let body = response
            .text()
            .await?;
        let user_details = helpers::parse_user_details(&body)?;
        
        Ok(user_details)
    }
    
    /// Accepts an offer. 
    pub async fn accept_offer(
        &self,
        tradeofferid: TradeOfferId,
        partner: SteamID,
    ) -> Result<AcceptedOffer, Error> {
        #[derive(Serialize)]
        struct AcceptOfferParams {
            sessionid: String,
            serverid: u32,
            #[serde(with = "serialize::string")]
            tradeofferid: TradeOfferId,
            captcha: &'static str,
            #[serde(serialize_with = "serialize::steamid_as_string")]
            partner: SteamID,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = self.get_uri(&format!("/tradeoffer/{tradeofferid}"));
        let params = AcceptOfferParams {
            sessionid,
            tradeofferid,
            partner,
            serverid: 1,
            captcha: "",
        };
        let uri = self.get_uri(&format!("/tradeoffer/{tradeofferid}/accept"));
        let response = self.client.post(&uri)
            .header(REFERER, referer)
            .form(&params)
            .send()
            .await?;
        let body: AcceptedOffer = parses_response(response).await?;
        
        Ok(body)
    }
    
    /// Declines an offer. 
    pub async fn decline_offer(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<TradeOfferId, Error> {
        #[derive(Serialize)]
        struct DeclineOfferParams {
            sessionid: String,
        }
        
        #[derive(Deserialize, Debug)]
        struct Response {
            #[serde(with = "serialize::string")]
            tradeofferid: TradeOfferId,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = self.get_uri(&format!("/tradeoffer/{tradeofferid}"));
        let uri = self.get_uri(&format!("/tradeoffer/{tradeofferid}/decline"));
        let response = self.client.post(&uri)
            .header(REFERER, referer)
            .form(&DeclineOfferParams {
                sessionid,
            })
            .send()
            .await?;
        let body: Response = parses_response(response).await?;
        
        Ok(body.tradeofferid)
    }
    
    /// Cancels an offer. 
    pub async fn cancel_offer(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<TradeOfferId, Error> {
        #[derive(Serialize)]
        struct CancelOfferParams {
            sessionid: String,
        }
        
        #[derive(Deserialize, Debug)]
        struct Response {
            #[serde(with = "serialize::string")]
            tradeofferid: TradeOfferId,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = self.get_uri(&format!("/tradeoffer/{tradeofferid}"));
        let uri = self.get_uri(&format!("/tradeoffer/{tradeofferid}/cancel"));
        let response = self.client.post(&uri)
            .header(REFERER, referer)
            .form(&CancelOfferParams {
                sessionid,
            })
            .send()
            .await?;
        let body: Response = parses_response(response).await?;
        
        Ok(body.tradeofferid)
    }
    
    /// Gets a user's inventory using the old endpoint.
    pub async fn get_inventory_old(
        &self,
        steamid: SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<Asset>, Error> { 
        #[derive(Serialize)]
        struct Query<'a> {
            l: &'a str,
            start: Option<u64>,
            trading: bool,
        }
        
        let mut responses: Vec<GetInventoryOldResponse> = Vec::new();
        let mut start: Option<u64> = None;
        let sid = u64::from(steamid);
        let uri = self.get_uri(&format!("/profiles/{sid}/inventory/json/{appid}/{contextid}"));
        let referer = self.get_uri(&format!("/profiles/{sid}/inventory"));
        
        loop {
            let response = self.client.get(&uri)
                .header(REFERER, &referer)
                .query(&Query {
                    l: self.language.api_language_code(),
                    trading: tradable_only,
                    start,
                })
                .send()
                .await?;
            let body: GetInventoryOldResponse = parses_response(response).await?;
            
            if !body.success {
                return Err(Error::ResponseUnsuccessful);
            } else if body.more_items {
                // shouldn't occur, but we wouldn't want to call this endlessly if it does...
                if body.more_start == start {
                    return Err(Error::MalformedResponse);
                }
                
                start = body.more_start;
                responses.push(body);
            } else {
                responses.push(body);
                break;
            }
        }
        
        let mut inventory = Vec::new();
        
        for body in responses {
            for item in body.assets.values() {
                let classinfo = body.descriptions.get(&(item.classid, item.instanceid))
                    .ok_or_else(|| Error::MissingClassInfo(MissingClassInfoError {
                        appid,
                        classid: item.classid,
                        instanceid: item.instanceid,
                    }))?;
                
                inventory.push(Asset {
                    appid,
                    contextid,
                    assetid: item.assetid,
                    amount: item.amount,
                    missing: false,
                    classinfo: Arc::clone(classinfo),
                });
            }
        }
        
        Ok(inventory)
    }
    
    /// Gets a user's inventory.
    pub async fn get_inventory(
        &self,
        steamid: SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<Asset>, Error> {
        get_inventory(&GetInventoryOptions {
            client: &self.client,
            steamid,
            appid,
            contextid,
            tradable_only,
            language: self.language,
        }).await
    }
    
    /// Gets a user's inventory which includes `app_data` using the `GetAssetClassInfo` API.
    pub async fn get_inventory_with_classinfos(
        &self,
        steamid: SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<Asset>, Error> { 
        #[derive(Serialize)]
        struct Query<'a> {
            l: &'a str,
            count: u32,
            start_assetid: Option<u64>,
        }
        
        let mut responses: Vec<GetInventoryResponseIgnoreDescriptions> = Vec::new();
        let mut start_assetid: Option<u64> = None;
        let sid = u64::from(steamid);
        let uri = self.get_uri(&format!("/inventory/{sid}/{appid}/{contextid}"));
        let referer = self.get_uri(&format!("/profiles/{sid}/inventory"));
        
        loop {
            let response = self.client.get(&uri)
                .header(REFERER, &referer)
                .query(&Query {
                    l: self.language.api_language_code(),
                    count: 2000,
                    start_assetid,
                })
                .send()
                .await?;
            let body: GetInventoryResponseIgnoreDescriptions = parses_response(response).await?;
            
            if !body.success {
                return Err(Error::ResponseUnsuccessful);
            } else if body.more_items {
                // shouldn't occur, but we wouldn't want to call this endlessly if it does...
                if body.last_assetid == start_assetid {
                    return Err(Error::MalformedResponse);
                }
                
                start_assetid = body.last_assetid;
                responses.push(body);
            } else {
                responses.push(body);
                break;
            }
        }
        
        let mut inventory = Vec::new();
        let items = responses
            .into_iter()
            .flat_map(|response| response.assets)
            .collect::<Vec<_>>();
        let classes = items
            .iter()
            .map(|item| (item.appid, item.classid, item.instanceid))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let map = self.get_asset_classinfos(&classes).await?;
        
        for item in items {
            let classinfo = map.get(&(appid, item.classid, item.instanceid))
                .ok_or_else(|| Error::MissingClassInfo(MissingClassInfoError {
                    appid,
                    classid: item.classid,
                    instanceid: item.instanceid,
                }))?;
            
            if tradable_only && !classinfo.tradable {
                continue;
            }
            
            inventory.push(Asset {
                appid,
                contextid,
                assetid: item.assetid,
                amount: item.amount,
                missing: false,
                classinfo: Arc::clone(classinfo),
            });
        }
        
        Ok(inventory)
    }
}

impl From<SteamTradeOfferAPIBuilder> for SteamTradeOfferAPI {
    fn from(builder: SteamTradeOfferAPIBuilder) -> Self {
        if !builder.data_directory.exists() {
            std::fs::create_dir_all(&builder.data_directory).ok();
        }
        
        let cookies = builder.cookies
            .unwrap_or_else(|| Arc::new(Jar::default()));
        let client = builder.client
            .unwrap_or_else(|| get_default_middleware(
                Arc::clone(&cookies),
                builder.user_agent,
            ));
        let classinfo_cache = builder.classinfo_cache.unwrap_or_default();
        
        Self {
            client,
            cookies: Arc::clone(&cookies),
            api_key: builder.api_key,
            language: builder.language,
            classinfo_cache,
            data_directory: builder.data_directory,
            sessionid: Arc::new(std::sync::RwLock::new(None)),
        }
    }
}