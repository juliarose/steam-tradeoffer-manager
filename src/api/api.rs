use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    time::Duration
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
    helpers::{get_default_middleware, parses_response},
};
use super::{
    raw,
    helpers::{
        parse_receipt_script,
        from_raw_trade_offer,
        from_raw_receipt_asset,
    },
    response::{
        GetTradeOffersResponseBody,
        GetTradeOffersResponse,
        GetInventoryResponse,
        GetInventoryOldResponse,
        GetAssetClassInfoResponse,
    }
};
use async_recursion::async_recursion;
use async_std::task::sleep;
use serde::{Deserialize, Serialize};
use serde_qs;
use reqwest::cookie::Jar;
use url::{Url, ParseError};
use reqwest::header::REFERER;
use lazy_regex::{regex_captures, regex_is_match};

const HOSTNAME: &'static str = "https://steamcommunity.com";
const API_HOSTNAME: &'static str = "https://api.steampowered.com";
const USER_AGENT_STRING: &'static str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36";
const ONE_YEAR_SECS: u64 = 31536000;

#[derive(Debug)]
pub struct SteamTradeOfferAPI {
    client: Client,
    pub key: String,
    pub cookies: Arc<Jar>,
    pub language: String,
    pub steamid: SteamID,
    pub identity_secret: Option<String>,
    pub sessionid: Arc<RwLock<Option<String>>>,
    pub classinfo_cache: Arc<RwLock<ClassInfoCache>>,
}

impl SteamTradeOfferAPI {
    
    pub fn new(
        cookies: Arc<Jar>,
        steamid: SteamID,
        key: String,
        language: String,
        identity_secret: Option<String>,
        classinfo_cache: Arc<RwLock<ClassInfoCache>>,
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
            let pathname: String = match offer.id {
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
                tradeofferid_countered: &offer.id,
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
    
    pub async fn get_trade_offers(
        &self,
        filter: &OfferFilter,
        historical_cutoff: &Option<ServerTime>,
    ) -> Result<Vec<response::trade_offer::TradeOffer>, Error> {
        let mut responses = Vec::new();
        let offers = self.get_trade_offers_request(&mut responses, filter, historical_cutoff, None).await?;
        
        Ok(offers)
    }
    
    pub async fn get_receipt(
        &self,
        trade_id: &TradeId,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        fn collect_classes(raw_assets: &Vec<raw::RawReceiptAsset>) -> Vec<ClassInfoClass> {
            let mut classes_set: HashSet<ClassInfoClass> = HashSet::new();

            for item in raw_assets {
                classes_set.insert((item.appid, item.classid, item.instanceid));
            }
            
            classes_set.into_iter().collect()
        }
        
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
                    let classes = collect_classes(&raw_assets);
                    let _ = self.get_asset_classinfos(&classes).await?;
                    let mut classinfo_cache = self.classinfo_cache.write().unwrap();
                    let assets = raw_assets
                        .into_iter()
                        .map(|asset| from_raw_receipt_asset(asset, &mut classinfo_cache))
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
        classes: &Vec<ClassInfoAppClass>,
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
        println!("get {}", uri);
        let response = self.client.get(&uri)
            .query(&query)
            .send()
            .await?;
        let body: GetAssetClassInfoResponse = parses_response(response).await?;
        let classinfos = body.result;
        
        classinfo_cache_helpers::save_classinfos(appid, &classinfos).await;
        
        let inserted = self.classinfo_cache
            .write()
            .unwrap()
            .insert_classinfos(appid, &classinfos)?;

        Ok(inserted)
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
            maps.push(self.get_app_asset_classinfos_chunk(appid, &chunk.to_vec()).await?);
        }
        
        Ok(maps)
    }
    
    pub async fn get_asset_classinfos(
        &self,
        classes: &Vec<ClassInfoClass>,
    ) -> Result<ClassInfoMap, Error> {
        let mut apps: HashMap<AppId, Vec<ClassInfoAppClass>> = HashMap::new();
        let mut map = HashMap::new();
        
        {
            let results = classinfo_cache_helpers::load_classinfos(classes).await;
            let mut classinfo_cache = self.classinfo_cache.write().unwrap();
            
            for (class, classinfo) in results.into_iter().flatten() {
                classinfo_cache.insert(class, classinfo);
            }
            
            for (appid, classid, instanceid) in classes {
                let class = (*appid, *classid, *instanceid);

                match classinfo_cache.get_classinfo(&class) {
                    Some(classinfo) => {
                        map.insert(class, classinfo);
                    },
                    None => {
                        match apps.get_mut(&appid) {
                            Some(classes) => {
                                classes.push((*classid, *instanceid));
                            },
                            None => {
                                let classes = vec![(*classid, *instanceid)];
                                
                                apps.insert(*appid, classes);
                            },
                        }
                    }
                };
            }
            
            // drop the write lock
        }
        
        for (appid, classes) in apps {
            for maps in self.get_app_asset_classinfos(appid, classes).await? {
                for (class, classinfo) in maps {
                    map.insert(class, Arc::clone(&classinfo));
                }
            }
        }
        
        Ok(map)
    }

    #[async_recursion]
    async fn get_trade_offers_request<'a, 'b>(
        &'a self,
        responses: &'b mut Vec<GetTradeOffersResponseBody>,
        filter: &OfferFilter,
        historical_cutoff: &Option<ServerTime>,
        cursor: Option<u32>,
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

        fn collect_classes(offers: &Vec<raw::RawTradeOffer>) -> Vec<ClassInfoClass> {
            let mut classes_set: HashSet<ClassInfoClass> = HashSet::new();

            for offer in offers {
                for item in &offer.items_to_give {
                    classes_set.insert((item.appid, item.classid, item.instanceid));
                }

                for item in &offer.items_to_receive {
                    classes_set.insert((item.appid, item.classid, item.instanceid));
                }
            }
            
            classes_set.into_iter().collect()
        }

        let time_historical_cutoff: u64 = match historical_cutoff {
            Some(cutoff) => cutoff.timestamp() as u64,
            None => get_system_time() + ONE_YEAR_SECS,
        };
        let uri = self.get_api_url("IEconService", "GetTradeOffers", 1);
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
        
        if next_cursor > Some(0) {
            responses.push(body.response);
    
            Ok(self.get_trade_offers_request(responses, filter, historical_cutoff, next_cursor).await?)
        } else {
            responses.push(body.response);
            
            let mut response_offers = Vec::new();
            
            for response in responses {
                response_offers.append(&mut response.trade_offers_received);
                response_offers.append(&mut response.trade_offers_sent);
            }

            let classes = collect_classes(&response_offers);
            let _ = self.get_asset_classinfos(&classes).await?;
            let mut classinfo_cache = self.classinfo_cache.write().unwrap();
            let offers = response_offers
                .into_iter()
                .map(|offer| from_raw_trade_offer(offer, &mut classinfo_cache))
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(offers)
        }
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
    ) -> Result<(), Error> {
        #[derive(Serialize, Debug)]
        struct Form<'a> {
            key: &'a str,
            tradeofferid: TradeOfferId,
        }

        let uri = self.get_api_url("IEconService", "DeclineTradeOffer", 1);
        let _response = self.client.post(&uri)
            .form(&Form {
                key: &self.key,
                tradeofferid,
            })
            .send()
            .await?;
        // let body: GetInventoryResponse = parses_response(response).await?;

        Ok(())
    }
    
    pub async fn cancel_offer(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<(), Error> {
        #[derive(Serialize, Debug)]
        struct Form<'a> {
            key: &'a str,
            tradeofferid: TradeOfferId,
        }

        let uri = self.get_api_url("IEconService", "CancelTradeOffer", 1);
        let _response = self.client.post(&uri)
            .form(&Form {
                key: &self.key,
                tradeofferid,
            })
            .send()
            .await?;
        // let body: GetInventoryResponse = parses_response(response).await?;
        
        Ok(())
    }
    
    #[async_recursion]
    async fn get_inventory_old_request(
        &self,
        responses: &mut Vec<GetInventoryOldResponse>,
        start: Option<u64>,
        steamid: &SteamID,
        appid: u32,
        contextid: u32,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> { 
        #[derive(Serialize, Debug)]
        struct Query<'a> {
            l: &'a str,
            start: Option<u64>,
        }
        
        let sid = u64::from(*steamid);
        let uri = self.get_uri(&format!("/profiles/{}/inventory/json/{}/{}", sid, appid, contextid));
        let referer = self.get_uri(&format!("/profiles/{}/inventory", sid));
        let response = self.client.get(&uri)
            .header(REFERER, referer)
            .query(&Query {
                l: &self.language,
                start,
            })
            .send()
            .await?;
        let body: GetInventoryOldResponse = parses_response(response).await?;
        
        if !body.success {
            Err(Error::Response("Bad response".into()))
        } else if body.more_items {
            // shouldn't occur, but we wouldn't want to call this endlessly if it does...
            if body.more_start == start {
                return Err(Error::Response("Bad response".into()));
            }
            
            let start = body.more_start.clone();
            
            responses.push(body);
            
            Ok(self.get_inventory_old_request(responses, start, steamid, appid, contextid, tradable_only).await?)
        } else {
            responses.push(body);
            
            let mut inventory: Inventory = Vec::new();
            
            for body in responses {
                for (_, item) in &body.assets {
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
    }
    
    #[async_recursion]
    async fn get_inventory_request(
        &self,
        responses: &mut Vec<GetInventoryResponse>,
        start_assetid: Option<u64>,
        steamid: &SteamID,
        appid: u32,
        contextid: u32,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> { 
        #[derive(Serialize, Debug)]
        struct Query<'a> {
            l: &'a str,
            count: u32,
            start_assetid: Option<u64>,
        }
        
        let sid = u64::from(*steamid);
        let uri = self.get_uri(&format!("/inventory/{}/{}/{}", sid, appid, contextid));
        let referer = self.get_uri(&format!("/profiles/{}/inventory", sid));
        let response = self.client.get(&uri)
            .header(REFERER, referer)
            .query(&Query {
                l: &self.language,
                count: 5000,
                start_assetid,
            })
            .send()
            .await?;
        let body: GetInventoryResponse = parses_response(response).await?;
        
        if !body.success {
            Err(Error::Response("Bad response".into()))
        } else if body.more_items {
            // shouldn't occur, but we wouldn't want to call this endlessly if it does...
            if body.last_assetid == start_assetid {
                return Err(Error::Response("Bad response".into()));
            }
            
            // space out requests
            sleep(Duration::from_secs(1)).await;
            
            Ok(self.get_inventory_request(responses, body.last_assetid, steamid, appid, contextid, tradable_only).await?)
        } else {
            responses.push(body);
            
            let mut inventory: Inventory = Vec::new();
            
            for body in responses {
                for item in &body.assets {
                    if let Some(classinfo) = body.descriptions.get(&(item.classid, item.instanceid)) {
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
    }
    
    pub async fn get_inventory_old(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        let responses = &mut Vec::new();
        let inventory = self.get_inventory_old_request(
            responses,
            None,
            steamid,
            appid,
            contextid,
            tradable_only
        ).await?;
        
        Ok(inventory)
    }
    
    pub async fn get_inventory(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        let responses = &mut Vec::new();
        let inventory = self.get_inventory_request(
            responses,
            None,
            steamid,
            appid,
            contextid,
            tradable_only
        ).await?;
        
        Ok(inventory)
    }
}