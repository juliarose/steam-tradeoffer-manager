mod response;
mod response_wrappers;
mod helpers;

pub use response::*;

use response_wrappers::*;
use crate::response::*;
use reqwest_middleware::ClientWithMiddleware;
use std::{path::PathBuf, collections::{HashMap, HashSet}, sync::{Arc, RwLock, Mutex}};
use crate::{
    SteamID,
    time::ServerTime,
    types::*,
    serializers::{string, steamid_as_string},
    helpers::parses_response,
    error::{Error, MissingClassInfoError},
    classinfo_cache::{ClassInfoCache, helpers as classinfo_cache_helpers},
    request::{NewTradeOffer, NewTradeOfferItem},
};
use serde::{Deserialize, Serialize};
use url::{Url, ParseError};
use reqwest::{cookie::Jar, header::REFERER};
use lazy_regex::{regex_captures, regex_is_match};

/// The underlying API.for ['crate::SteamTradeOfferManager'].
#[derive(Debug, Clone)]
pub struct SteamTradeOfferAPI {
    /// The client for making requests.
    client: Client,
    /// The API key.
    pub key: String,
    /// The cookies to make requests with. Since the requests are made with the provided client, 
    /// the cookies should be the same as what the client uses.
    pub cookies: Arc<Jar>,
    /// The language for descriptions.
    pub language: String,
    /// The SteamID of the account.
    pub steamid: SteamID,
    /// The session ID.
    pub sessionid: Arc<RwLock<Option<String>>>,
    /// The cache for setting and getting [`ClassInfo`] data.
    pub classinfo_cache: Arc<Mutex<ClassInfoCache>>,
    /// The directory to store [`ClassInfo`] data.
    pub data_directory: PathBuf,
}

impl SteamTradeOfferAPI {
    pub const HOSTNAME: &str = "https://steamcommunity.com";
    pub const API_HOSTNAME: &str = "https://api.steampowered.com";
    
    /// Creates a new [`SteamTradeOfferAPI`].
    pub fn new(
        client: ClientWithMiddleware,
        cookies: Arc<Jar>,
        steamid: SteamID,
        key: String,
        language: String,
        classinfo_cache: Arc<Mutex<ClassInfoCache>>,
        data_directory: PathBuf,
    ) -> Self {
        Self {
            client,
            key,
            steamid,
            language,
            cookies: Arc::clone(&cookies),
            sessionid: Arc::new(RwLock::new(None)),
            classinfo_cache,
            data_directory,
        }
    }
    
    fn get_uri(
        &self,
        pathname: &str,
    ) -> String {
        format!("{}{}", Self::HOSTNAME, pathname)
    }
    
    fn get_api_url(
        &self,
        interface: &str,
        method: &str,
        version: usize,
    ) -> String {
        format!("{}/{}/{}/v{}", Self::API_HOSTNAME, interface, method, version)
    }
    
    /// Sets cookies.
    pub fn set_cookies(&self, cookies: &Vec<String>) -> Result<(), ParseError> {
        let url = Self::HOSTNAME.parse::<Url>()?;
        
        for cookie_str in cookies {
            self.cookies.add_cookie_str(cookie_str, &url);
        }
        
        Ok(())
    }
    
    /// Sets session.
    pub fn set_session(
        &self,
        sessionid: &str,
        cookies: &Vec<String>,
    ) -> Result<(), ParseError> {
        let mut sessionid_write = self.sessionid.write().unwrap();
        
        *sessionid_write = Some(sessionid.to_string());
        
        self.set_cookies(cookies)?;
        
        Ok(())
    }
    
    /// Sends an offer.
    pub async fn send_offer(
        &self,
        offer: &NewTradeOffer,
        counter_tradeofferid: Option<TradeOfferId>,
    ) -> Result<SentOffer, Error> {
        #[derive(Serialize, Debug)]
        struct OfferFormUser<'b> {
            assets: &'b Vec<NewTradeOfferItem>,
            currency: Vec<Currency>,
            ready: bool,
        }

        #[derive(Serialize, Debug)]
        struct OfferForm<'b> {
            newversion: bool,
            version: u32,
            me: OfferFormUser<'b>,
            them: OfferFormUser<'b>,
        }

        #[derive(Serialize, Debug)]
        struct TradeOfferCreateParams<'b> {
            #[serde(skip_serializing_if = "Option::is_none")]
            trade_offer_access_token: &'b Option<String>,
        }

        #[derive(Serialize, Debug)]
        struct SendOfferParams<'b> {
            sessionid: String,
            serverid: u32,
            json_tradeoffer: String,
            tradeoffermessage: &'b Option<String>,
            captcha: &'static str,
            trade_offer_create_params: String,
            tradeofferid_countered: &'b Option<u64>,
            #[serde(serialize_with = "steamid_as_string")]
            partner: &'b SteamID,
        }
        
        #[derive(Serialize, Debug)]
        struct RefererParams<'b> {
            partner: u32,
            token: &'b Option<String>,
        }
        
        let num_items: usize = offer.items_to_give.len() + offer.items_to_receive.len();

        if num_items == 0 {
            return Err(Error::Parameter("Cannot send an empty offer"));
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
            })?;
            
            self.get_uri(&format!(
                "/tradeoffer/{}?{}",
                pathname,
                qs_params
            ))
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
        let uri = self.get_uri(&format!("/trade/{}/receipt", trade_id));
        let response = self.client.get(&uri)
            .send()
            .await?;
        let body = response.text().await?;
        
        if let Some((_, message)) = regex_captures!(r#"<div id="error_msg">\s*([^<]+)\s*</div>"#, &body) {
           Err(Error::Response(message.trim().into()))
        } else if let Some((_, script)) = regex_captures!(r#"(var oItem;[\s\S]*)</script>"#, &body) {
            match helpers::parse_receipt_script(script) {
                Ok(raw_assets) => {
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
                },
                Err(error) => {
                    Err(Error::Response(error.into()))
                },
            }
        } else if regex_is_match!(r#"\{"success": ?false\}"#, &body) {
            Err(Error::NotLoggedIn)
        } else {
            Err(Error::MalformedResponse)
        }
    }
    
    /// Gets a chunk of [`response::class_info::ClassInfo`] data.
    pub async fn get_app_asset_classinfos_chunk(
        &self,
        appid: AppId,
        classes: &[ClassInfoAppClass],
    ) -> Result<ClassInfoMap, Error> {
        let query = {
            let mut query = vec![
                ("key".to_string(), self.key.to_string()),
                ("appid".to_string(), appid.to_string()),
                ("language".to_string(), self.language.clone()),
                ("class_count".to_string(), classes.len().to_string()),
            ];
            
            for (i, (classid, instanceid)) in classes.iter().enumerate() {
                query.push((format!("classid{}", i), classid.to_string()));
                
                if let Some(instanceid) = instanceid {
                    query.push((format!("instanceid{}", i), instanceid.to_string()));
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
            .filter_map(|((classid, instanceid), classinfo_string)| {
                serde_json::from_str::<ClassInfo>(&classinfo_string)
                    // ignore classinfos that failed parsed
                    .ok()
                    .map(|classinfo| {
                        (
                            (appid, classid, instanceid),
                            Arc::new(classinfo),
                        )
                    })
            })
            .collect::<HashMap<_, _>>();
        
        self.classinfo_cache.lock().unwrap().insert_classinfos(&classinfos);

        Ok(classinfos)
    }
    
    /// Gets [`response::class_info::ClassInfo`] data for appid.
    async fn get_app_asset_classinfos(
        &self,
        appid: AppId,
        classes: Vec<ClassInfoAppClass>,
    ) -> Result<Vec<ClassInfoMap>, Error> {
        let chuck_size = 100;
        let chunks = classes.chunks(chuck_size);
        let mut maps = Vec::with_capacity(chunks.len());
        
        for chunk in chunks {
            maps.push(self.get_app_asset_classinfos_chunk(appid, chunk).await?);
        }
        
        Ok(maps)
    }
    
    /// Gets [`response::class_info::ClassInfo`] data for the given classes.
    pub async fn get_asset_classinfos(
        &self,
        classes: &Vec<ClassInfoClass>,
    ) -> Result<ClassInfoMap, Error> {
        let mut apps: HashMap<AppId, Vec<ClassInfoAppClass>> = HashMap::new();
        let mut map: HashMap<ClassInfoClass, Arc<ClassInfo>> = HashMap::new();
        let mut needed: HashSet<&ClassInfoClass> = HashSet::from_iter(classes.iter());
        
        if classes.is_empty() {
            return Ok(map);
        }
        
        {
            {
                // check memory for caches
                let mut classinfo_cache = self.classinfo_cache.lock().unwrap();
                
                needed = needed
                    .into_iter()
                    .filter(|class| {
                        if let Some(classinfo) = classinfo_cache.get(class) {
                            map.insert(**class, classinfo);
                            // we don't need it
                            false
                        } else {
                            true
                        }
                    })
                    .collect::<HashSet<_>>();
                
                // drop the lock
            }
            
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
                    let mut classinfo_cache = self.classinfo_cache.lock().unwrap();
                    
                    for (class, classinfo) in results {
                        let classinfo = Arc::new(classinfo);
                        
                        needed.remove(&class);
                        classinfo_cache.insert(class, Arc::clone(&classinfo));
                        map.insert(class, classinfo);
                    }
            
                    // drop the lock
                }
            }
            
            for (appid, classid, instanceid) in needed {
                match apps.get_mut(appid) {
                    Some(classes) => {
                        classes.push((*classid, *instanceid));
                    },
                    None => {
                        let classes = vec![(*classid, *instanceid)];
                        
                        apps.insert(*appid, classes);
                    },
                }
            }
        }
        
        for (appid, classes) in apps {
            for maps in self.get_app_asset_classinfos(appid, classes).await? {
                for (class, classinfo) in maps {
                    map.insert(class, classinfo);
                }
            }
        }
        
        Ok(map)
    }
    
    /// Gets trade offer data before any descriptions are added.
    pub async fn get_raw_trade_offers(
        &self,
        active_only: bool,
        historical_only: bool,
        get_sent_offers: bool,
        get_received_offers: bool,
        get_descriptions: bool,
        historical_cutoff: Option<ServerTime>,
    ) -> Result<(Vec<response::RawTradeOffer>, Option<ClassInfoMap>), Error> {
        #[derive(Serialize, Debug)]
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
        
        let mut cursor = None;
        let time_historical_cutoff = historical_cutoff
            .map(|cutoff| cutoff.timestamp() as u64);
        let uri = self.get_api_url("IEconService", "GetTradeOffers", 1);
        let mut offers = Vec::new();
        let mut descriptions = Vec::new();
        
        loop {
            let response = self.client.get(&uri)
                .query(&Form {
                    key: &self.key,
                    language: &self.language,
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
        offers: Vec<response::RawTradeOffer>,
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
        #[derive(Serialize, Debug)]
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
        let response = self.client.get(&uri)
            .query(&Form {
                key: &self.key,
                tradeofferid,
            })
            .send()
            .await?;
        let body: Response = parses_response(response).await?;
        
        Ok(body.response.offer)
    }
    
    pub async fn get_trade_history(
        &self,
        max_trades: u32,
        start_after_time: Option<u32>,
        start_after_tradeid: Option<TradeId>,
        navigating_back: bool,
        get_descriptions: bool,
        include_failed: bool,
        include_total: bool,
    ) -> Result<(Vec<RawTrade>, bool), Error> {
        #[derive(Serialize, Debug)]
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

        #[derive(Deserialize, Debug)]
        pub struct Response {
            #[serde(default)]
            more: bool,
            trades: Vec<RawTrade>,
        }
        
        let uri = self.get_api_url("IEconService", "GetTradeHistory", 1);
        let response = self.client.get(&uri)
            .query(&Form {
                key: &self.key,
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
        let body: Response = parses_response(response).await?;
        
        Ok((body.trades, body.more))
    }
    
    /// Gets escrow details for user.
    pub async fn get_user_details(
        &self,
        partner: &SteamID,
        tradeofferid: Option<TradeOfferId>,
        token: &Option<String>,
    ) -> Result<UserDetails, Error> {
        #[derive(Serialize, Debug)]
        struct Params<'b> {
            partner: u32,
            token: &'b Option<String>,
        }

        fn get_days(group: Option<(&str, &str)>) -> u32 {
            match group {
                Some((_, days_str)) => {
                    match days_str.parse::<u32>() {
                        Ok(days) => days,
                        Err(_e) => 0,
                    }
                },
                None => 0,
            }
        }
        
        let uri = {
            let pathname: String = match tradeofferid {
                Some(id) => id.to_string(),
                None => String::from("new"),
            };
            let qs_params = serde_qs::to_string(&Params {
                partner: partner.account_id(),
                token,
            })?;
            
            self.get_uri(&format!(
                "/tradeoffer/{}?{}",
                pathname,
                qs_params
            ))
        };
        let response = self.client.get(&uri)
            .send()
            .await?;
        let body = response
            .text()
            .await?;
        
        println!("{}", body);
        if regex_is_match!(r#"\n\W*<script type="text/javascript">\W*\r?\n?(\W*var g_rgAppContextData[\s\S]*)</script>"#, &body) {
            let my_escrow_days = get_days(
                regex_captures!(r#"var g_daysMyEscrow = (\d+);"#, &body)
            );
            let them_escrow_days = get_days(
                regex_captures!(r#"var g_daysTheirEscrow = (\d+);"#, &body)
            );

            Ok(UserDetails {
                my_escrow_days,
                them_escrow_days,
            })
        } else {
            Err(Error::MalformedResponse)
        }
    }
    
    /// Accepts an offer. 
    pub async fn accept_offer(
        &self,
        tradeofferid: TradeOfferId,
        partner: &SteamID,
    ) -> Result<AcceptedOffer, Error> {
        #[derive(Serialize, Debug)]
        struct AcceptOfferParams<'b> {
            sessionid: String,
            serverid: u32,
            #[serde(with = "string")]
            tradeofferid: TradeOfferId,
            captcha: &'static str,
            #[serde(serialize_with = "steamid_as_string")]
            partner: &'b SteamID,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = self.get_uri(&format!("/tradeoffer/{}", tradeofferid));
        let params = AcceptOfferParams {
            sessionid,
            tradeofferid,
            partner,
            serverid: 1,
            captcha: "",
        };
        let uri = self.get_uri(&format!("/tradeoffer/{}/accept", tradeofferid));
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
        #[derive(Serialize, Debug)]
        struct DeclineOfferParams {
            sessionid: String,
        }
        
        #[derive(Deserialize, Debug)]
        struct Response {
            #[serde(with = "string")]
            tradeofferid: TradeOfferId,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = self.get_uri(&format!("/tradeoffer/{}", tradeofferid));
        let uri = self.get_uri(&format!("/tradeoffer/{}/decline", tradeofferid));
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
        #[derive(Serialize, Debug)]
        struct CancelOfferParams {
            sessionid: String,
        }
        
        #[derive(Deserialize, Debug)]
        struct Response {
            #[serde(with = "string")]
            tradeofferid: TradeOfferId,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone()
            .ok_or(Error::NotLoggedIn)?;
        let referer = self.get_uri(&format!("/tradeoffer/{}", tradeofferid));
        let uri = self.get_uri(&format!("/tradeoffer/{}/cancel", tradeofferid));
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
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<Asset>, Error> { 
        #[derive(Serialize, Debug)]
        struct Query<'a> {
            l: &'a str,
            start: Option<u64>,
            trading: bool,
        }
        
        let mut responses: Vec<GetInventoryOldResponse> = Vec::new();
        let mut start: Option<u64> = None;
        let sid = u64::from(*steamid);
        let uri = self.get_uri(&format!("/profiles/{}/inventory/json/{}/{}", sid, appid, contextid));
        let referer = self.get_uri(&format!("/profiles/{}/inventory", sid));
        
        loop {
            let response = self.client.get(&uri)
                .header(REFERER, &referer)
                .query(&Query {
                    l: &self.language,
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
        
        let mut inventory: Inventory = Vec::new();
        
        for body in responses {
            for item in body.assets.values() {
                let classinfo = body.descriptions.get(&(item.classid, item.instanceid))
                    .ok_or_else(|| Error::MissingClassInfo(MissingClassInfoError {
                        appid,
                        classid: item.classid,
                        instanceid: item.instanceid,
                    }))?;
                
                inventory.push(Asset {
                    classinfo: Arc::clone(classinfo),
                    appid,
                    contextid,
                    assetid: item.assetid,
                    amount: item.amount,
                });
            }
        }
        
        Ok(inventory)
    }
    
    /// Gets a user's inventory.
    pub async fn get_inventory(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<Asset>, Error> { 
        #[derive(Serialize, Debug)]
        struct Query<'a> {
            l: &'a str,
            count: u32,
            start_assetid: Option<u64>,
        }
        
        let mut responses: Vec<GetInventoryResponse> = Vec::new();
        let mut start_assetid: Option<u64> = None;
        let sid = u64::from(*steamid);
        let uri = self.get_uri(&format!("/inventory/{}/{}/{}", sid, appid, contextid));
        let referer = self.get_uri(&format!("/profiles/{}/inventory", sid));
        
        loop {
            let response = self.client.get(&uri)
                .header(REFERER, &referer)
                .query(&Query {
                    l: &self.language,
                    count: 2000,
                    start_assetid,
                })
                .send()
                .await?;
            let body: GetInventoryResponse = parses_response(response).await?;
            
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
        
        let mut inventory: Inventory = Vec::new();
        
        for body in responses {
            for item in &body.assets {
                let classinfo = body.descriptions.get(&(item.classid, item.instanceid))
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
                    classinfo: Arc::clone(classinfo),
                });
            }
        }
        
        Ok(inventory)
    }
    
    /// Gets a user's inventory with more detailed clasinfo data using the GetAssetClassInfo API.
    pub async fn get_inventory_with_classinfos(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<Asset>, Error> { 
        #[derive(Serialize, Debug)]
        struct Query<'a> {
            l: &'a str,
            count: u32,
            start_assetid: Option<u64>,
        }
        
        let mut responses: Vec<GetInventoryResponseIgnoreDescriptions> = Vec::new();
        let mut start_assetid: Option<u64> = None;
        let sid = u64::from(*steamid);
        let uri = self.get_uri(&format!("/inventory/{}/{}/{}", sid, appid, contextid));
        let referer = self.get_uri(&format!("/profiles/{}/inventory", sid));
        
        loop {
            let response = self.client.get(&uri)
                .header(REFERER, &referer)
                .query(&Query {
                    l: &self.language,
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
        
        let mut inventory: Inventory = Vec::new();
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
                classinfo: Arc::clone(classinfo),
            });
        }
        
        Ok(inventory)
    }
}