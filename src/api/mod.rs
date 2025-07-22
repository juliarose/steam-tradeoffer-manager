//! This is the underlying API for the manager. In most cases you should stick to using the
//! manager, but if you need more control over the requests, you can use this API directly.

pub mod response;
pub mod request;

mod builder;
mod response_wrappers;
mod helpers;

/// The default number of items to fetch per page when getting inventories.
pub(crate) const DEFAULT_GET_INVENTORY_PAGE_SIZE: u32 = 2000;

use response::*;
use response_wrappers::*;

pub use builder::SteamTradeOfferAPIBuilder;

use crate::SteamID;
use crate::helpers::get_default_client;
use crate::types::*;
use crate::response::*;
use crate::enums::{Language, GetUserDetailsMethod};
use crate::static_functions::get_inventory;
use crate::serialize;
use crate::helpers::{parses_response, generate_sessionid, extract_auth_data_from_cookies};
use crate::helpers::{COMMUNITY_HOSTNAME, WEB_API_HOSTNAME, CookiesData};
use crate::error::{Error, ParameterError, MissingClassInfoError, SetCookiesError};
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

/// Session data from cookies.
#[derive(Debug, Clone, Default)]
pub(crate) struct Session {
    /// The session ID.
    pub sessionid: Option<String>,
    /// The access token for trade offers.
    pub access_token: Option<String>,
}

/// The underlying API for interacting with Steam trade offers.
#[derive(Debug, Clone)]
pub struct SteamTradeOfferAPI {
    /// The API key.
    pub api_key: Option<String>,
    /// The access token for trade offers.
    pub(crate) session: Arc<RwLock<Session>>,
    /// The language for descriptions.
    pub language: Language,
    /// The number of items to fetch per page when getting inventories.
    pub get_inventory_page_size: u32,
    /// The client for making requests.
    client: HttpClient,
    /// The cookies to make requests with. Since the requests are made with the provided client, 
    /// the cookies should be the same as what the client uses.
    cookies: Arc<Jar>,
    /// The cache for setting and getting [`ClassInfo`] data.
    classinfo_cache: ClassInfoCache,
    /// The directory to store [`ClassInfo`] data.
    pub(crate) data_directory: PathBuf,
}

impl SteamTradeOfferAPI {
    /// Hostname for requests.
    const HOSTNAME: &'static str = COMMUNITY_HOSTNAME;
    /// Hostname for API requests.
    const API_HOSTNAME: &'static str = WEB_API_HOSTNAME;
    
    /// Builder for constructing a [`SteamTradeOfferAPI`].
    pub fn builder() -> SteamTradeOfferAPIBuilder {
        SteamTradeOfferAPIBuilder::new()
    }
    
    fn get_url(
        pathname: &str,
    ) -> String {
        format!("https://{}{pathname}", Self::HOSTNAME)
    }
    
    fn get_api_url(
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
        mut cookies: Vec<String>,
    ) -> Result<(), SetCookiesError> {
        let CookiesData {
            sessionid,
            access_token,
            ..
        } = extract_auth_data_from_cookies(&cookies)?;
        let sessionid = if let Some(sessionid) = sessionid {
            sessionid
        } else {
            // the cookies don't contain a sessionid
            let sessionid = generate_sessionid();
            
            cookies.push(format!("sessionid={sessionid}"));
            sessionid
        };
        // Should not panic since the URL is hardcoded.
        let url = format!("https://{}", Self::HOSTNAME).parse::<Url>()
            .unwrap_or_else(|error| panic!("URL could not be parsed from {}: {}", Self::HOSTNAME, error));
        
        *self.session.write().unwrap() = Session {
            sessionid: Some(sessionid),
            access_token: Some(access_token),
        };
        
        for cookie_str in &cookies {
            self.cookies.add_cookie_str(cookie_str, &url);
        }
        
        Ok(())
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
        
        let num_items = offer.items_to_give.len() + offer.items_to_receive.len();
        
        if num_items == 0 {
            return Err(Error::Parameter(ParameterError::EmptyOffer));
        }
        
        let sessionid = self.session.read().unwrap().sessionid.clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = {
            let pathname: String = match &counter_tradeofferid {
                Some(id) => id.to_string(),
                None => String::from("new"),
            };
            
            
            helpers::offer_referer_url(&pathname, offer.partner, &offer.token.as_deref())?
        };
        let params: SendOfferParams<'_> = {
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
        let uri = Self::get_url("/tradeoffer/new/send");
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
        let uri = Self::get_url(&format!("/trade/{trade_id}/receipt"));
        let response = self.client.get(&uri)
            .send()
            .await?;
        let body = response.text().await?;
        
        if let Some((_, message)) = regex_captures!(r#"<div id="error_msg">\s*([^<]+)\s*</div>"#, &body) {
           return Err(Error::UnexpectedResponse(message.trim().into()));
        }
        
        if let Some((_, script)) = regex_captures!(r#"(var oItem;[\s\S]*)</script>"#, &body) {
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
            
            return Ok(assets);
        }
        
        if regex_is_match!(r#"\{"success": ?false\}"#, &body) {
            return Err(Error::NotLoggedIn);
        }
        
        Err(Error::MalformedResponseWithBody("Page does not include receipt script.", body))
    }
    
    /// Gets a chunk of [`ClassInfo`] data.
    async fn get_app_asset_classinfos_chunk(
        &self,
        appid: AppId,
        classes: &[ClassInfoAppClass],
    ) -> Result<ClassInfoMap, Error> {
        let query = {
            let key = self.api_key.as_ref();
            let access_token = self.session.read().unwrap().access_token.clone();
            
            if key.is_none() && access_token.is_none() {
                return Err(ParameterError::MissingApiKeyOrAccessToken.into());
            }
            
            let mut query = Vec::new();
            
            if let Some(access_token) = access_token {
                // No need to provide the key if we have an access token.
                query.push(("access_token".to_string(), access_token));
            } else {
                // unwrap is safe here since we checked for the presence of the key above.
                query.push(("key".to_string(), key.unwrap().into()));
            }
            
            query.push(("appid".to_string(), appid.to_string()));
            query.push(("language".to_string(), self.language.web_api_language_code().to_string()));
            query.push(("class_count".to_string(), classes.len().to_string()));
            
            for (i, (classid, instanceid)) in classes.iter().enumerate() {
                query.push((format!("classid{i}"), classid.to_string()));
                
                if let Some(instanceid) = instanceid {
                    query.push((format!("instanceid{i}"), instanceid.to_string()));
                }
            }
            
            query
        };
        let uri = Self::get_api_url("ISteamEconomy", "GetAssetClassInfo", 1);
        let response = self.client.get(&uri)
            .query(&query)
            .send()
            .await?;
        let body: GetAssetClassInfoResponse = parses_response(response).await?;
        // Convert the classinfos into a map.
        let (
            classinfos,
            classinfos_raw,
        ): (
            HashMap<_, _>,
            Vec<_>,
        ) = body.result
            .into_iter()
            // Sometimes Steam returns empty classinfo data.
            // We just ignore them until they are successfully fetched.
            .filter_map(|((classid, instanceid), classinfo_raw)| {
                let classinfo = serde_json::from_str::<ClassInfo>(classinfo_raw.get())
                    // Ignores invalid or empty classinfo data.
                    .ok()?;
                // We return a pair so that we have a deserialized version to return from the
                // method and a raw version to save to the file system. We do not need to clone
                // data since we are keeping the boxed raw values to send to the tokio task. This
                // should be quite efficient.
                let pair = (
                    ((appid, classid, instanceid), Arc::new(classinfo)),
                    ((classid, instanceid), classinfo_raw),
                );
                
                Some(pair)
            })
            .unzip();
        // Save the classinfos to the filesystem.
        // This spawns a tokio task which will save the classinfos to the filesystem in the
        // background so that this method does not need to await on it.
        let _handle = classinfo_cache_helpers::save_classinfos(
            appid,
            classinfos_raw,
            &self.data_directory,
        );
        
        // And return the classinfos.
        Ok(classinfos)
    }
    
    /// Gets [`ClassInfo`] data for `appid`.
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
        classes: &[ClassInfoClass],
    ) -> Result<ClassInfoMap, Error> {
        if classes.is_empty() {
            return Ok(Default::default());
        }
        
        let mut apps: HashMap<AppId, Vec<ClassInfoAppClass>> = HashMap::new();
        // Check memory for caches.
        let (
            mut map,
            misses,
        ) = self.classinfo_cache.get_map(classes);
        let mut needed = HashSet::from_iter(misses);
        
        if !needed.is_empty() {
            // Check filesystem for caches.
            let results = classinfo_cache_helpers::load_classinfos(
                &needed,
                &self.data_directory,
            ).await
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            
            if !results.is_empty() {
                let mut inserts = HashMap::with_capacity(results.len());
                
                for (class, classinfo) in results {
                    let classinfo = Arc::new(classinfo);
                    
                    needed.remove(&class);
                    inserts.insert(class, Arc::clone(&classinfo));
                }
                
                // Insert the classinfos into the cache.
                self.classinfo_cache.insert_map(inserts.clone());
                map.extend(inserts);
            }
        }
        
        let mut cache_map = HashMap::with_capacity(needed.len());
        
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
            for app_map in self.get_app_asset_classinfos(appid, classes).await? {
                cache_map.extend(app_map.clone());
                map.extend(app_map);
            }
        }
        
        if !cache_map.is_empty() {
            // Insert newly obtained classinfos into the cache for later use.
            self.classinfo_cache.insert_map(cache_map);
        }
        
        Ok(map)
    }
    
    /// Gets trade offer data before any descriptions are added. The 2nd part of the tuple are the
    /// descriptions from the response if `get_descriptions` was set. These can be combined with
    /// the offers using the `map_raw_trade_offers_with_descriptions` method.
    pub async fn get_raw_trade_offers(
        &self,
        options: &request::GetTradeOffersOptions,
    ) -> Result<(Vec<response::RawTradeOffer>, Option<ClassInfoMap>), Error> {
        #[derive(Serialize)]
        struct Form<'a, 'b> {
            key: Option<&'a String>,
            access_token: Option<&'b String>,
            language: &'a str,
            active_only: bool,
            historical_only: bool,
            get_sent_offers: bool,
            get_received_offers: bool,
            get_descriptions: bool,
            time_historical_cutoff: Option<u64>,
            cursor: Option<u32>,
        }
        
        let request::GetTradeOffersOptions {
            active_only,
            historical_only,
            get_sent_offers,
            get_received_offers,
            get_descriptions,
            historical_cutoff,
        } = options;
        let uri = Self::get_api_url("IEconService", "GetTradeOffers", 1);
        let mut key = self.api_key.as_ref();
        let access_token = self.session.read().unwrap().access_token.clone();
        
        if key.is_none() && access_token.is_none() {
            return Err(ParameterError::MissingApiKeyOrAccessToken.into());
        }
        
        if access_token.is_some() {
            // No need to provide the key if we have an access token.
            key = None;
        }
        
        let mut cursor = None;
        let time_historical_cutoff = historical_cutoff
            .map(|cutoff| cutoff.timestamp() as u64);
        let mut offers = Vec::new();
        let mut descriptions = Vec::new();
        
        loop {
            let response = self.client.get(&uri)
                .query(&Form {
                    key,
                    access_token: access_token.as_ref(),
                    language: self.language.web_api_language_code(),
                    active_only: *active_only,
                    historical_only: *historical_only,
                    get_sent_offers: *get_sent_offers,
                    get_received_offers: *get_received_offers,
                    get_descriptions: *get_descriptions,
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
                    .any(|offer| offer.time_created < *historical_cutoff);
                
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
    
    /// Combines trade offers with their descriptions using the cache and the Steam Web API. 
    /// Ignores offers with missing descriptions.
    pub async fn map_raw_trade_offers(
        &self,
        offers: Vec<response::RawTradeOffer>,
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
            .collect::<Vec<_>>();
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
        options: &request::GetTradeOffersOptions,
    ) -> Result<Vec<TradeOffer>, Error> {
        let (raw_offers, _descriptions) = self.get_raw_trade_offers(options).await?;
        let offers = self.map_raw_trade_offers(raw_offers).await?;
        
        Ok(offers)
    }
    
    /// Gets a trade offer.
    pub async fn get_trade_offer(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<response::RawTradeOffer, Error> {
        #[derive(Serialize)]
        struct Form<'a, 'b> {
            key: Option<&'a String>,
            acccess_token: Option<&'b String>,
            tradeofferid: TradeOfferId,
        }
        
        #[derive(Deserialize)]
        struct Body {
            offer: response::RawTradeOffer,
        }
        
        #[derive(Deserialize)]
        struct Response {
            response: Body,
        }
        
        let uri = Self::get_api_url("IEconService", "GetTradeOffer", 1);
        let mut key = self.api_key.as_ref();
        let access_token = self.session.read().unwrap().access_token.clone();
        
        if key.is_none() && access_token.is_none() {
            return Err(ParameterError::MissingApiKeyOrAccessToken.into());
        }
        
        if access_token.is_some() {
            // No need to provide the key if we have an access token.
            key = None;
        }
        
        let response = self.client.get(&uri)
            .query(&Form {
                key,
                acccess_token: access_token.as_ref(),
                tradeofferid,
            })
            .send()
            .await?;
        let body: Response = parses_response(response).await?;
        
        Ok(body.response.offer)
    }
    
    /// Gets trade history.
    pub async fn get_trade_history(
        &self,
        options: &GetTradeHistoryOptions,
    ) -> Result<Trades, Error> {
        let body = self.get_trade_history_request(request::GetTradeHistoryRequestOptions{
            max_trades: options.max_trades,
            start_after_time: options.start_after_time,
            start_after_tradeid: options.start_after_tradeid,
            navigating_back: options.navigating_back,
            get_descriptions: true,
            include_failed: options.include_failed,
            include_total: true,
        }).await?;
        
        if body.trades.is_empty() {
            return Ok(Trades {
                trades: Vec::new(),
                more: body.more,
                // Should always be present since include_total was passed.
                total_trades: body.total_trades.unwrap_or_default(),
            });
        }
        
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
    ) -> Result<response::RawTrades, Error> {
        let body = self.get_trade_history_request(request::GetTradeHistoryRequestOptions{
            max_trades: options.max_trades,
            start_after_time: options.start_after_time,
            start_after_tradeid: options.start_after_tradeid,
            navigating_back: options.navigating_back,
            get_descriptions: false,
            include_failed: options.include_failed,
            include_total: true,
        }).await?;
        
        Ok(response::RawTrades {
            trades: body.trades,
            more: body.more,
            // Should always be present since include_total was passed.
            total_trades: body.total_trades.unwrap_or_default(),
        })
    }
    
    async fn get_trade_history_request(
        &self,
        options: request::GetTradeHistoryRequestOptions,
    ) -> Result<GetTradeHistoryResponseBody, Error> {
        #[derive(Serialize)]
        struct Form<'a, 'b> {
            key: Option<&'a String>,
            acccess_token: Option<&'b String>,
            max_trades: u32,
            start_after_time: Option<u32>,
            start_after_tradeid: Option<TradeId>,
            navigating_back: bool,
            get_descriptions: bool,
            include_failed: bool,
            include_total: bool,
        }
        
        let request::GetTradeHistoryRequestOptions {
            max_trades,
            start_after_time,
            start_after_tradeid,
            navigating_back,
            get_descriptions,
            include_failed,
            include_total,
        } = options;
        // Convert the datetime to a UNIX timestamp.
        let start_after_time = start_after_time
            .map(|time| time.timestamp() as u32);
        let mut key = self.api_key.as_ref();
        let access_token = self.session.read().unwrap().access_token.clone();
        
        if key.is_none() && access_token.is_none() {
            return Err(ParameterError::MissingApiKeyOrAccessToken.into());
        }
        
        if access_token.is_some() {
            // No need to provide the key if we have an access token.
            key = None;
        }
        
        let uri = Self::get_api_url("IEconService", "GetTradeHistory", 1);
        let response = self.client.get(&uri)
            .query(&Form {
                key,
                acccess_token: access_token.as_ref(),
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
        let uri = {
            let method = method.into();
            let pathname = method.pathname();
            
            
            helpers::offer_referer_url(&pathname, partner, &method.token())?
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
        
        let sessionid = self.session.read().unwrap().sessionid.clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = Self::get_url(&format!("/tradeoffer/{tradeofferid}"));
        let params = AcceptOfferParams {
            sessionid,
            tradeofferid,
            partner,
            serverid: 1,
            captcha: "",
        };
        let uri = Self::get_url(&format!("/tradeoffer/{tradeofferid}/accept"));
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
        
        #[derive(Deserialize)]
        struct Response {
            #[serde(with = "serialize::string")]
            tradeofferid: TradeOfferId,
        }
        
        let sessionid = self.session.read().unwrap().sessionid.clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = Self::get_url(&format!("/tradeoffer/{tradeofferid}"));
        let uri = Self::get_url(&format!("/tradeoffer/{tradeofferid}/decline"));
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
        
        #[derive(Deserialize)]
        struct Response {
            #[serde(with = "serialize::string")]
            tradeofferid: TradeOfferId,
        }
        
        let sessionid = self.session.read().unwrap().sessionid.clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = Self::get_url(&format!("/tradeoffer/{tradeofferid}"));
        let uri = Self::get_url(&format!("/tradeoffer/{tradeofferid}/cancel"));
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
        let uri = Self::get_url(&format!("/profiles/{sid}/inventory/json/{appid}/{contextid}"));
        let referer = Self::get_url(&format!("/profiles/{sid}/inventory"));
        
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
            }
            
            if body.more_items {
                // shouldn't occur, but we wouldn't want to call this endlessly if it does...
                if body.more_start == start {
                    return Err(Error::MalformedResponse("Pagination cursor is the same as the previous response."));
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
    /// 
    /// The number of items to fetch per request can be set using with
    /// [`crate::TradeOfferManagerBuilder::get_inventory_page_size`].
    pub async fn get_inventory(
        &self,
        steamid: SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<Asset>, Error> {
        let access_token = self.session.read().unwrap().access_token.clone();
        
        get_inventory(&GetInventoryOptions {
            client: &self.client,
            steamid,
            appid,
            contextid,
            tradable_only,
            language: self.language,
            page_size: self.get_inventory_page_size,
            access_token,
        }).await
    }
    
    /// Gets a user's inventory which includes `app_data` using the `GetAssetClassInfo` API.
    /// 
    /// The number of items to fetch per request can be set using with
    /// [`crate::TradeOfferManagerBuilder::get_inventory_page_size`].
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
            access_token: Option<&'a String>,
        }
        
        let mut responses: Vec<GetInventoryResponseIgnoreDescriptions> = Vec::new();
        let mut start_assetid: Option<u64> = None;
        let access_token = self.session.read().unwrap().access_token.clone();
        let sid = u64::from(steamid);
        let uri = Self::get_url(&format!("/inventory/{sid}/{appid}/{contextid}"));
        let referer = Self::get_url(&format!("/profiles/{sid}/inventory"));
        
        loop {
            let response = self.client.get(&uri)
                .header(REFERER, &referer)
                .query(&Query {
                    l: self.language.api_language_code(),
                    count: self.get_inventory_page_size,
                    start_assetid,
                    access_token: access_token.as_ref(),
                })
                .send()
                .await?;
            let body: GetInventoryResponseIgnoreDescriptions = parses_response(response).await?;
            
            if !body.success {
                return Err(Error::ResponseUnsuccessful);
            }
            
            if body.more_items {
                // shouldn't occur, but we wouldn't want to call this endlessly if it does...
                if body.last_assetid == start_assetid {
                    return Err(Error::MalformedResponse("Pagination cursor is the same as the previous response."));
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
        
        let cookies = builder.cookie_jar
            .unwrap_or_default();
        let client = builder.client
            .unwrap_or_else(|| get_default_client(
                Arc::clone(&cookies),
                builder.user_agent,
            ));
        let classinfo_cache = builder.classinfo_cache.unwrap_or_default();
        let session = Session {
            access_token: builder.access_token,
            sessionid: None,
        };
        
        Self {
            client,
            cookies,
            api_key: builder.api_key,
            session: Arc::new(std::sync::RwLock::new(session)),
            language: builder.language,
            get_inventory_page_size: builder.get_inventory_page_size,
            classinfo_cache,
            data_directory: builder.data_directory,
        }
    }
}
