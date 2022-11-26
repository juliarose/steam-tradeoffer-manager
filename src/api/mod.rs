mod raw;
mod api_response;
mod helpers;

use helpers::{
    parse_receipt_script,
    from_raw_trade_offer,
    from_raw_receipt_asset,
};
use api_response::{
    GetTradeOffersResponseBody,
    GetTradeOffersResponse,
    GetInventoryResponse,
    GetInventoryOldResponse,
    GetAssetClassInfoResponse,
};
use std::{
    path::PathBuf,
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock, Mutex},
};
use crate::{
    error::Error,
    enums::OfferFilter,
    SteamID,
    time::{ServerTime, get_system_time},
    classinfo_cache::{ClassInfoCache, helpers as classinfo_cache_helpers},
    types::{
        ClassInfoMap,
        ClassInfoAppClass,
        ClassInfoClass,
        Inventory,
        TradeOfferId,
        AppId,
        ContextId,
        TradeId,
        Client,
    },
    response,
    request::{self, serializers::steamid_as_string},
    serializers::string,
    helpers::{get_default_middleware, get_proxied_middleware ,parses_response},
};
use serde::{Deserialize, Serialize};
use reqwest::{Proxy, cookie::Jar};
use url::{Url, ParseError};
use reqwest::header::REFERER;
use lazy_regex::{regex_captures, regex_is_match};

const HOSTNAME: &str = "https://steamcommunity.com";
const API_HOSTNAME: &str = "https://api.steampowered.com";
const USER_AGENT_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36";
const ONE_YEAR_SECS: u64 = 31536000;

/// Type of request.
#[derive(Debug)]
enum RequestType {
    Cookies,
    Proxied(Proxy),
}

#[derive(Debug)]
pub struct SteamTradeOfferAPI {
    client: Client,
    pub key: String,
    pub cookies: Arc<Jar>,
    pub language: String,
    pub steamid: SteamID,
    pub identity_secret: Option<String>,
    pub sessionid: Arc<RwLock<Option<String>>>,
    pub classinfo_cache: Arc<Mutex<ClassInfoCache>>,
    pub data_directory: PathBuf,
}

impl SteamTradeOfferAPI {
    pub fn new(
        cookies: Arc<Jar>,
        steamid: SteamID,
        key: String,
        language: String,
        identity_secret: Option<String>,
        classinfo_cache: Arc<Mutex<ClassInfoCache>>,
        data_directory: PathBuf,
    ) -> Self {
        Self {
            client: get_default_middleware(Arc::clone(&cookies), USER_AGENT_STRING),
            key,
            steamid,
            identity_secret,
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
        format!("{}{}", HOSTNAME, pathname)
    }

    fn get_api_url(
        &self,
        interface: &str,
        method: &str,
        version: usize,
    ) -> String {
        format!("{}/{}/{}/v{}", API_HOSTNAME, interface, method, version)
    }
    
    fn set_cookies(&self, cookies: &Vec<String>) -> Result<(), ParseError> {
        let url = HOSTNAME.parse::<Url>()?;
        
        for cookie_str in cookies {
            self.cookies.add_cookie_str(cookie_str, &url);
        }
        
        Ok(())
    }
    
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
    
    pub async fn send_offer(
        &self,
        offer: &request::trade_offer::NewTradeOffer,
        counter_tradeofferid: Option<TradeOfferId>,
    ) -> Result<response::sent_offer::SentOffer, Error> {
        #[derive(Serialize, Debug)]
        struct OfferFormUser<'b> {
            assets: &'b Vec<request::trade_offer::Item>,
            currency: Vec<response::Currency>,
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
        struct SendOfferParams<'a, 'b> {
            sessionid: &'a String,
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
        
        let sessionid = self.sessionid.read().unwrap().clone();
        
        if sessionid.is_none() {
            return Err(Error::NotLoggedIn);
        }
        
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
                // presence of sessionid was checked above - unwrap is safe here
                sessionid: &sessionid.unwrap(),
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
        let body: response::sent_offer::SentOffer = parses_response(response).await?;
        
        Ok(body)
    }
    
    pub async fn get_receipt(
        &self,
        trade_id: &TradeId,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        let uri = self.get_uri(&format!("/trade/{}/receipt", trade_id));
        let response = self.client.get(&uri)
            .send()
            .await?;
        let body = response.text().await?;
        
        if let Some((_, message)) = regex_captures!(r#"<div id="error_msg">\s*([^<]+)\s*</div>"#, &body) {
           Err(Error::Response(message.trim().into()))
        } else if let Some((_, script)) = regex_captures!(r#"(var oItem;[\s\S]*)</script>"#, &body) {
            match parse_receipt_script(script) {
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
                        .map(|asset| from_raw_receipt_asset(asset, &map))
                        .collect::<Result<Vec<_>, _>>()?;
                    
                    Ok(assets)
                },
                Err(error) => {
                    Err(Error::Response(error.into()))
                }
            }
        } else if regex_is_match!(r#"\{"success": ?false\}"#, &body) {
            Err(Error::NotLoggedIn)
        } else {
            Err(Error::Response("Unexpected response".into()))
        }
    }
    
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
            .map(|((classid, instanceid), classinfo_string)| {
                serde_json::from_str::<response::ClassInfo>(&classinfo_string)
                    .map(|classinfo| {
                        (
                            (appid, classid, instanceid),
                            Arc::new(classinfo),
                        )
                    })
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        
        self.classinfo_cache.lock().unwrap().insert_classinfos(&classinfos);

        Ok(classinfos)
    }
    
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
    
    pub async fn get_asset_classinfos(
        &self,
        classes: &Vec<ClassInfoClass>,
    ) -> Result<ClassInfoMap, Error> {
        let mut apps: HashMap<AppId, Vec<ClassInfoAppClass>> = HashMap::new();
        let mut map: HashMap<ClassInfoClass, Arc<response::ClassInfo>> = HashMap::new();
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
                        if let Some(classinfo) = classinfo_cache.get_classinfo(class) {
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

    pub async fn get_trade_offers(
        &self,
        filter: &OfferFilter,
        historical_cutoff: &Option<ServerTime>,
    ) -> Result<Vec<response::trade_offer::TradeOffer>, Error> {
        #[derive(Serialize, Debug)]
        struct Form<'a> {
            key: &'a str,
            language: &'a str,
            active_only: bool,
            historical_only: bool,
            get_sent_offers: bool,
            get_received_offers: bool,
            get_descriptions: bool,
            time_historical_cutoff: u64,
            cursor: Option<u32>,
        }
        
        let mut cursor = None;
        let mut responses: Vec<GetTradeOffersResponseBody> = Vec::new();
        let time_historical_cutoff: u64 = match historical_cutoff {
            Some(cutoff) => cutoff.timestamp() as u64,
            None => get_system_time() + ONE_YEAR_SECS,
        };
        let uri = self.get_api_url("IEconService", "GetTradeOffers", 1);
        
        loop {
            let response = self.client.get(&uri)
                .query(&Form {
                    key: &self.key,
                    language: &self.language,
                    active_only: *filter == OfferFilter::ActiveOnly,
                    historical_only: *filter == OfferFilter::HistoricalOnly,
                    get_sent_offers: true,
                    get_received_offers: true,
                    get_descriptions: false,
                    time_historical_cutoff,
                    cursor,
                })
                .send()
                .await?;
            let body: GetTradeOffersResponse = parses_response(response).await?;
            let next_cursor = body.response.next_cursor;
            
            responses.push(body.response);
            
            if next_cursor > Some(0) {
                cursor = next_cursor;
            } else {
                break;
            }
        }
        
        let mut response_offers = Vec::new();
        
        for mut response in responses {
            response_offers.append(&mut response.trade_offers_received);
            response_offers.append(&mut response.trade_offers_sent);
        }

        let classes = response_offers
            .iter()
            .flat_map(|offer| {
                offer.items_to_give
                    .iter()
                    .chain(offer.items_to_receive.iter())
                    .map(|item| (item.appid, item.classid, item.instanceid))
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let map = self.get_asset_classinfos(&classes).await?;
        let offers = response_offers
            .into_iter()
            .map(|offer| from_raw_trade_offer(offer, &map))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(offers)
    }

    pub async fn get_trade_offer(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<raw::RawTradeOffer, Error> {
        #[derive(Serialize, Debug)]
        struct Form<'a> {
            key: &'a str,
            tradeofferid: TradeOfferId,
        }

        #[derive(Deserialize, Debug)]
        struct Body {
            offer: raw::RawTradeOffer,
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

    pub async fn get_user_details(
        &self,
        tradeofferid: &Option<TradeOfferId>,
        partner: &SteamID,
        token: &Option<String>,
    ) -> Result<response::user_details::UserDetails, Error> {
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
        
        if regex_is_match!(r#"/\n\W*<script type="text/javascript">\W*\r?\n?(\W*var g_rgAppContextData[\s\S]*)</script>"#, &body) {
            let my_escrow = get_days(regex_captures!(r#"var g_daysMyEscrow = (\d+);"#, &body));
            let them_escrow = get_days(regex_captures!(r#"var g_daysTheirEscrow = (\d+);"#, &body));

            Ok(response::user_details::UserDetails {
                my_escrow,
                them_escrow,
            })
        } else {
            Err(Error::Response("Malformed response".into()))
        }
    }

    pub async fn accept_offer(
        &self,
        tradeofferid: TradeOfferId,
        partner: &SteamID,
    ) -> Result<response::accepted_offer::AcceptedOffer, Error> {
        #[derive(Serialize, Debug)]
        struct AcceptOfferParams<'a, 'b> {
            sessionid: &'a String,
            serverid: u32,
            #[serde(with = "string")]
            tradeofferid: TradeOfferId,
            captcha: &'static str,
            #[serde(serialize_with = "steamid_as_string")]
            partner: &'b SteamID,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone();
        
        if sessionid.is_none() {
            return Err(Error::NotLoggedIn);
        }
        
        let referer = self.get_uri(&format!("/tradeoffer/{}", tradeofferid));
        let params = AcceptOfferParams {
            sessionid: &sessionid.unwrap(),
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
        let body: response::accepted_offer::AcceptedOffer = parses_response(response).await?;
        
        Ok(body)
    }

    pub async fn decline_offer(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<TradeOfferId, Error> {
        #[derive(Serialize, Debug)]
        struct DeclineOfferParams<'a> {
            sessionid: &'a String,
        }
        
        #[derive(Deserialize, Debug)]
        struct Response {
            #[serde(with = "string")]
            tradeofferid: TradeOfferId,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone();
        
        if sessionid.is_none() {
            return Err(Error::NotLoggedIn);
        }
        
        let referer = self.get_uri(&format!("/tradeoffer/{}", tradeofferid));
        let uri = self.get_uri(&format!("/tradeoffer/{}/decline", tradeofferid));
        let response = self.client.post(&uri)
            .header(REFERER, referer)
            .form(&DeclineOfferParams {
                sessionid: &sessionid.unwrap(),
            })
            .send()
            .await?;
        let body: Response = parses_response(response).await?;
        
        Ok(body.tradeofferid)
    }
    
    pub async fn cancel_offer(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<TradeOfferId, Error> {
        #[derive(Serialize, Debug)]
        struct CancelOfferParams<'a> {
            sessionid: &'a String,
        }
        
        #[derive(Deserialize, Debug)]
        struct Response {
            #[serde(with = "string")]
            tradeofferid: TradeOfferId,
        }
        
        let sessionid = self.sessionid.read().unwrap().clone();
        
        if sessionid.is_none() {
            return Err(Error::NotLoggedIn);
        }
        
        let referer = self.get_uri(&format!("/tradeoffer/{}", tradeofferid));
        let uri = self.get_uri(&format!("/tradeoffer/{}/cancel", tradeofferid));
        let response = self.client.post(&uri)
            .header(REFERER, referer)
            .form(&CancelOfferParams {
                sessionid: &sessionid.unwrap(),
            })
            .send()
            .await?;
        let body: Response = parses_response(response).await?;
        
        Ok(body.tradeofferid)
    }

    async fn get_inventory_old_request(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
        request_type: &RequestType,
    ) -> Result<Vec<response::asset::Asset>, Error> { 
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
            let response = match request_type {
                RequestType::Cookies => {
                    self.client.get(&uri)
                        .header(REFERER, &referer)
                        .query(&Query {
                            l: &self.language,
                            trading: tradable_only,
                            start,
                        })
                        .send()
                        .await
                },
                RequestType::Proxied(proxy) => {
                    get_proxied_middleware(
                        USER_AGENT_STRING,
                        proxy.clone(),
                    ).get(&uri)
                        .header(REFERER, &referer)
                        .query(&Query {
                            l: &self.language,
                            trading: tradable_only,
                            start,
                        })
                        .send()
                        .await
                },
            }?;
            let body: GetInventoryOldResponse = parses_response(response).await?;
            
            if !body.success {
                return Err(Error::Response("Bad response".into()));
            } else if body.more_items {
                // shouldn't occur, but we wouldn't want to call this endlessly if it does...
                if body.more_start == start {
                    return Err(Error::Response("Bad response".into()));
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
                if let Some(classinfo) = body.descriptions.get(&(item.classid, item.instanceid)) {
                    inventory.push(response::asset::Asset {
                        classinfo: Arc::clone(classinfo),
                        appid,
                        contextid,
                        assetid: item.assetid,
                        amount: item.amount,
                    });
                } else {
                    let instanceid =  item.instanceid.unwrap_or(0);
                    
                    return Err(Error::Response(
                        format!("Missing descriptions for item {}:{}", item.classid, instanceid)
                    ));
                }
            }
        }
        
        Ok(inventory)
    }
    
    async fn get_inventory_request(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
        request_type: &RequestType,
    ) -> Result<Vec<response::asset::Asset>, Error> { 
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
            let response = match request_type {
                RequestType::Cookies => {
                    self.client.get(&uri)
                        .header(REFERER, &referer)
                        .query(&Query {
                            l: &self.language,
                            count: 2000,
                            start_assetid,
                        })
                        .send()
                        .await
                },
                RequestType::Proxied(proxy) => {
                    get_proxied_middleware(
                        USER_AGENT_STRING,
                        proxy.clone(),
                    ).get(&uri)
                        .header(REFERER, &referer)
                        .query(&Query {
                            l: &self.language,
                            count: 2000,
                            start_assetid,
                        })
                        .send()
                        .await
                },
            }?;
            let body: GetInventoryResponse = parses_response(response).await?;
            
            if !body.success {
                return Err(Error::Response("Bad response".into()));
            } else if body.more_items {
                // shouldn't occur, but we wouldn't want to call this endlessly if it does...
                if body.last_assetid == start_assetid {
                    return Err(Error::Response("Bad response".into()));
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
                if let Some(classinfo) = body.descriptions.get(&(item.classid, item.instanceid)) {
                    if tradable_only && !classinfo.tradable {
                        continue;
                    }
                    
                    inventory.push(response::asset::Asset {
                        appid: item.appid,
                        contextid: item.contextid,
                        assetid: item.assetid,
                        amount: item.amount,
                        classinfo: Arc::clone(classinfo),
                    });
                } else {
                    let instanceid =  item.instanceid.unwrap_or(0);
                    
                    return Err(Error::Response(
                        format!("Missing descriptions for item {}:{}", item.classid, instanceid)
                    ));
                }
            }
        }
        
        Ok(inventory)
    }
    
    async fn get_inventory_with_classinfos_request(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
        request_type: &RequestType,
    ) -> Result<Vec<response::asset::Asset>, Error> { 
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
            let response = match request_type {
                RequestType::Cookies => {
                    self.client.get(&uri)
                        .header(REFERER, &referer)
                        .query(&Query {
                            l: &self.language,
                            count: 2000,
                            start_assetid,
                        })
                        .send()
                        .await
                },
                RequestType::Proxied(proxy) => {
                    get_proxied_middleware(
                        USER_AGENT_STRING,
                        proxy.clone(),
                    ).get(&uri)
                        .header(REFERER, &referer)
                        .query(&Query {
                            l: &self.language,
                            count: 2000,
                            start_assetid,
                        })
                        .send()
                        .await
                },
            }?;
            let body: GetInventoryResponse = parses_response(response).await?;
            
            if !body.success {
                return Err(Error::Response("Bad response".into()));
            } else if body.more_items {
                // shouldn't occur, but we wouldn't want to call this endlessly if it does...
                if body.last_assetid == start_assetid {
                    return Err(Error::Response("Bad response".into()));
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
            if let Some(classinfo) = map.get(&(item.appid, item.classid, item.instanceid)) {
                if tradable_only && !classinfo.tradable {
                    continue;
                }
                
                inventory.push(response::asset::Asset {
                    appid: item.appid,
                    contextid: item.contextid,
                    assetid: item.assetid,
                    amount: item.amount,
                    classinfo: Arc::clone(classinfo),
                });
            } else {
                let instanceid =  item.instanceid.unwrap_or(0);
                
                return Err(Error::Response(
                    format!("Missing descriptions for item {}:{}", item.classid, instanceid)
                ));
            }
        }
        
        Ok(inventory)
    }
    
    pub async fn get_inventory_old(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        self.get_inventory_old_request(
            steamid,
            appid,
            contextid,
            tradable_only,
            &RequestType::Cookies,
        ).await
    }
    
    pub async fn get_inventory(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        self.get_inventory_request(
            steamid,
            appid,
            contextid,
            tradable_only,
            &RequestType::Cookies,
        ).await
    }
    
    pub async fn get_inventory_with_classinfos(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        self.get_inventory_with_classinfos_request(
            steamid,
            appid,
            contextid,
            tradable_only,
            &RequestType::Cookies,
        ).await
    }

    pub async fn get_inventory_proxied(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
        proxy: Proxy,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        self.get_inventory_request(
            steamid,
            appid,
            contextid,
            tradable_only,
            &RequestType::Proxied(proxy),
        ).await
    }
}