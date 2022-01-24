use crate::{
    APIError,
    Item,
    Currency,
    OfferFilter,
    TradeOfferState,
    ConfirmationMethod,
    ServerTime,
    classinfo_cache::{
        ClassInfoCache,
        save_classinfos
    },
    response::{
        self,
        ClassInfo,
        ClassInfoMap,
        ClassInfoAppClass,
        ClassInfoClass,
        ClassInfoAppMap,
        TradeOffer,
        UserDetails,
        Asset,
        Inventory,
        deserializers::{
            from_int_to_bool,
            to_classinfo_map,
            deserialize_classinfo_map_raw
        }
    },
    request,
    api_helpers::{
        get_default_middleware,
        parses_response
    },
    serializers::{
        string,
        option_string,
        option_str_to_number,
        steamid_as_string
    }
};
use chrono::serde::ts_seconds;
use async_recursion::async_recursion;
use deepsize::DeepSizeOf;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_qs;
use reqwest::{cookie::Jar, Url};
use reqwest_middleware::ClientWithMiddleware;
use reqwest::header::REFERER;
use steamid_ng::SteamID;
use std::sync::Arc;
use lazy_regex::{regex_captures, regex_is_match};
use async_std::task::sleep;
use itertools::Itertools;

const HOSTNAME: &'static str = "https://steamcommunity.com";
const API_HOSTNAME: &'static str = "https://api.steampowered.com";

pub struct SteamTradeOfferAPI {
    key: String,
    cookies: Arc<Jar>,
    client: ClientWithMiddleware,
    sessionid: Option<String>,
    classinfo_cache: ClassInfoCache,
}

impl SteamTradeOfferAPI {
    
    pub fn new(key: String) -> Self {
        let cookies = Arc::new(Jar::default());
        let mut classinfo_cache = ClassInfoCache::new();
        
        Self {
            key,
            cookies: Arc::clone(&cookies),
            client: get_default_middleware(Arc::clone(&cookies)),
            sessionid: None,
            classinfo_cache,
        }
    }
    
    fn get_uri(&self, pathname: &str) -> String {
        format!("{}{}", HOSTNAME, pathname)
    }

    fn get_api_url(&self, interface: &str, method: &str, version: usize) -> String {
        format!("{}/{}/{}/v{}", API_HOSTNAME, interface, method, version)
    }
    
    pub fn set_cookie(&mut self, cookie_str: &str) {
        if let Ok(url) = HOSTNAME.parse::<Url>() {
            self.cookies.add_cookie_str(cookie_str, &url);
            
            if let Some((_, sessionid)) = regex_captures!(r#"sessionid=([A-z0-9]+)"#, cookie_str) {
                self.sessionid = Some(String::from(sessionid));
            }
        }
    }
    
    pub fn set_cookies(&mut self, cookies: &Vec<String>) {
        for cookie_str in cookies {
            self.set_cookie(cookie_str)
        }
    }

    pub async fn send_offer<'a, 'b>(&self, offer: &'b request::CreateTradeOffer) -> Result<response::SentOffer, APIError> {
        #[derive(Serialize, Debug)]
        struct OfferFormUser<'b> {
            assets: &'b Vec<Item>,
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
            return Err(APIError::ParameterError("Cannot send an empty offer"));
        }
        
        let sessionid = match &self.sessionid {
            Some(sessionid) => sessionid,
            None => return Err(APIError::NotLoggedIn),
        };
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
                tradeofferid_countered: &offer.id,
            }
        };
        let uri = self.get_uri("/tradeoffer/new/send");
        let response = self.client.post(&uri)
            .header(REFERER, referer)
            .form(&params)
            .send()
            .await?;
        let body: response::SentOffer = parses_response(response).await?;
        
        Ok(body)
    }
    
    pub async fn get_trade_offers(&mut self) -> Result<Vec<TradeOffer>, APIError> {
        let mut responses = Vec::new();
        let offers = self.get_trade_offers_request(&mut responses, &OfferFilter::ActiveOnly, &None, None).await?;
        
        Ok(offers)
    }
    
    pub async fn get_app_asset_classinfos_chunk(&mut self, appid: u32, classes: &Vec<ClassInfoAppClass>) -> Result<ClassInfoMap, APIError> {
        let query = {
            let mut query = Vec::new();
            
            query.push(("key".to_string(), self.key.to_string()));
            query.push(("appid".to_string(), appid.to_string()));
            query.push(("language".to_string(), "english".to_string()));
            query.push(("class_count".to_string(), classes.len().to_string()));
            
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
        
        save_classinfos(appid, &classinfos).await;

        Ok(self.classinfo_cache.insert_classinfos(appid, &classinfos)?)
    }
    
    async fn get_app_asset_classinfos(&mut self, appid: u32, classes: Vec<ClassInfoAppClass>) -> Result<Vec<ClassInfoMap>, APIError> {
        let mut maps = Vec::new();
        
        for chunk in classes.chunks(100) {
            maps.push(self.get_app_asset_classinfos_chunk(appid, &chunk.to_vec()).await?);
        }
        
        Ok(maps)
    }
    
    pub async fn get_asset_classinfos(&mut self, classes: &Vec<ClassInfoClass>) -> Result<ClassInfoMap, APIError> {
        let mut apps: HashMap<u32, Vec<ClassInfoAppClass>> = HashMap::new();
        let mut map = HashMap::new();
        
        self.classinfo_cache.load_classes(classes).await;
        
        for (appid, classid, instanceid) in classes {
            let class = (*appid, *classid, *instanceid);

            match self.classinfo_cache.get_classinfo(&class) {
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
    async fn get_trade_offers_request<'a, 'b>(&'a mut self, responses: &'b mut Vec<GetTradeOffersResponseBody>, filter: &OfferFilter, historical_cutoff: &Option<ServerTime>, cursor: Option<u32>) -> Result<Vec<TradeOffer>, APIError> {
        #[derive(Serialize, Debug)]
        struct Form<'a, 'b> {
            key: &'a str,
            language: &'b str,
            get_sent_offers: bool,
            get_received_offers: bool,
            get_descriptions: bool,
            time_historical_cutoff: u64,
            cursor: Option<u32>,
        }

        let language = "english";
        let uri = self.get_api_url("IEconService", "GetTradeOffers", 1);
        let response = self.client.get(&uri)
            .query(&Form {
                key: &self.key,
                language,
                get_sent_offers: true,
                get_received_offers: true,
                get_descriptions: false,
                // todo
                time_historical_cutoff: 1642165779 * 2,
                cursor: None,
            })
            .send()
            .await?;
        let body: GetTradeOffersResponse = parses_response(response).await?;
        
        fn steamid_from_accountid(accountid: u32) -> SteamID {
            SteamID::new(
                accountid,
                steamid_ng::Instance::Desktop,
                steamid_ng::AccountType::Individual,
                steamid_ng::Universe::Public
            )
        }
        
        fn collect_items(assets: Vec<RawAsset>, cache: &ClassInfoCache) -> Result<Vec<Asset>, APIError> {
            let mut items = Vec::new();
            
            for asset in assets {
                if let Some(classinfo) = cache.get_classinfo(&(asset.appid, asset.classid, asset.instanceid)) {
                    items.push(Asset {
                        classinfo,
                        appid: asset.appid,
                        contextid: asset.contextid,
                        assetid: asset.assetid,
                        amount: asset.amount,
                    });
                } else {
                    let instanceid = match asset.instanceid {
                        Some(instanceid) => instanceid,
                        None => 0,
                    };
                    
                    return Err(APIError::ResponseError(format!("Missing descriptions for item {}:{}", asset.classid, instanceid).into()));
                }
            }
            
            Ok(items)
        }

        fn collect_classes(offers: &Vec<RawTradeOffer>) -> Vec<ClassInfoClass> {
            let mut classes_set: HashSet<ClassInfoClass> = HashSet::new();

            for offer in offers {
                for item in &offer.items_to_give {
                    classes_set.insert((item.appid, item.classid, item.instanceid));
                }

                for item in &offer.items_to_receive {
                    classes_set.insert((item.appid, item.classid, item.instanceid));
                }
            }
            
            let classes: Vec<_> = classes_set.into_iter().collect();
            
            classes
        }
        
        let next_cursor = body.response.next_cursor;

        if next_cursor > 0 {
            responses.push(body.response);
    
            Ok(self.get_trade_offers_request(responses, filter, historical_cutoff, Some(next_cursor)).await?)
        } else {
            responses.push(body.response);
            
            let mut offers: Vec<TradeOffer> = Vec::new();
            let mut response_offers = Vec::new();
            
            for response in responses {
                response_offers.append(&mut response.trade_offers_received);
                response_offers.append(&mut response.trade_offers_sent);
            }

            let classes = collect_classes(&response_offers);
            let _classinfos = self.get_asset_classinfos(&classes).await?;

            for offer in response_offers {
                let items_to_give = collect_items(offer.items_to_give, &self.classinfo_cache)?;
                let items_to_receive = collect_items(offer.items_to_receive, &self.classinfo_cache)?;
                
                offers.push(TradeOffer {
                    tradeofferid: offer.tradeofferid,
                    trade_offer_state: offer.trade_offer_state,
                    partner: steamid_from_accountid(offer.accountid_other),
                    message: offer.message,
                    items_to_give,
                    items_to_receive,
                    is_our_offer: offer.is_our_offer,
                    from_real_time_trade: offer.from_real_time_trade,
                    expiration_time: offer.expiration_time,
                    time_updated: offer.time_updated,
                    time_created: offer.time_created,
                    escrow_end_date: offer.escrow_end_date,
                    confirmation_method: offer.confirmation_method,
                });
            }
            
            Ok(offers)
        }
    }

    pub async fn get_trade_offer<'a>(&self, tradeofferid: u64) -> Result<RawTradeOffer, APIError> {
        #[derive(Serialize, Debug)]
        struct Form<'a> {
            key: &'a str,
            tradeofferid: u64,
        }

        #[derive(Deserialize, Debug)]
        struct Body {
            offer: RawTradeOffer,
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

    pub async fn get_user_details<'a, 'b>(&'a self, tradeofferid: Option<u64>, partner: &'b SteamID, token: &'b Option<String>) -> Result<UserDetails, APIError> {
        #[derive(Serialize, Debug)]
        struct Params<'b> {
            partner: u32,
            token: &'b Option<String>,
        }
        
        fn parse_days(days_str: &str) -> u32 {
            match days_str.parse::<u32>() {
                Ok(days) => days,
                Err(_e) => 0,
            }
        }
        
        let uri = {
            let pathname: String = match tradeofferid{
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
            let my_escrow = match regex_captures!(r#"var g_daysMyEscrow = (\d+);"#, &body) {
                Some((_, days_str)) => parse_days(days_str),
                None => 0,
            };
            let them_escrow = match regex_captures!(r#"var g_daysTheirEscrow = (\d+);"#, &body) {
                Some((_, days_str)) => parse_days(days_str),
                None => 0,
            };

            Ok(UserDetails {
                my_escrow,
                them_escrow,
            })
        } else {
            Err(APIError::ResponseError("Malformed response".into()))
        }
    }

    pub async fn decline_offer<'a>(&self, tradeofferid: u64) -> Result<(), APIError> {
        #[derive(Serialize, Debug)]
        struct Form<'a> {
            key: &'a str,
            tradeofferid: u64,
        }

        let uri = self.get_api_url("IEconService", "DeclineTradeOffer", 1);
        let response = self.client.post(&uri)
            .form(&Form {
                key: &self.key,
                tradeofferid,
            })
            .send()
            .await?;
        // let body: GetInventoryResponse = parses_response(response).await?;

        Ok(())
    }
    
    pub async fn cancel_offer<'a>(&self, tradeofferid: u64) -> Result<(), APIError> {
        #[derive(Serialize, Debug)]
        struct Form<'a> {
            key: &'a str,
            tradeofferid: u64,
        }

        let uri = self.get_api_url("IEconService", "CancelTradeOffer", 1);
        let response = self.client.post(&uri)
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
    async fn get_inventory_request(&mut self, responses: &mut Vec<GetInventoryResponse>, start_assetid: Option<u64>, steamid: &SteamID, appid: u32, contextid: u32, tradable_only: bool) -> Result<Inventory, APIError> { 
        #[derive(Serialize, Debug)]
        struct Query<'a> {
            l: &'a str,
            count: u32,
            start_assetid: Option<u64>,
        }

        fn collect_classes(items: &Vec<RawAsset>) -> Vec<ClassInfoClass> {
            let mut classes_set: HashSet<ClassInfoClass> = HashSet::new();

            for item in items {
                classes_set.insert((item.appid, item.classid, item.instanceid));
            }
            
            let classes: Vec<_> = classes_set.into_iter().collect();
            
            classes
        }
        
        let sid = u64::from(steamid.clone());
        let uri = self.get_uri(&format!("/inventory/{}/{}/{}", sid, appid, contextid));
        let referer = self.get_uri(&format!("/profiles/{}/inventory", sid));
        let language = "english";
        let response = self.client.get(&uri)
            .header(REFERER, referer)
            .query(&Query {
                l: language,
                count: 5000,
                start_assetid,
            })
            .send()
            .await?;
        let body: GetInventoryResponse = parses_response(response).await?;
        
        if !body.success {
            Err(APIError::ResponseError("Bad response".into()))
        } else if body.more_items {
            // shouldn't occur, but we wouldn't want to call this endlessly if it does...
            if body.last_assetid == start_assetid {
                return Err(APIError::ResponseError("Bad response".into()));
            }
            
            // space out requests
            sleep(Duration::from_secs(1)).await;
            
            Ok(self.get_inventory_request(responses, body.last_assetid, steamid, appid, contextid, tradable_only).await?)
        } else {
            responses.push(body);
            
            let mut inventory: Vec<Asset> = Vec::new();
            let items: Vec<RawAsset> = responses
                .iter()
                .map(|response| response.to_owned().assets)
                .flatten()
                .collect();
            let classes = collect_classes(&items);
            let _classinfos = self.get_asset_classinfos(&classes).await?;

            for item in &items {
                let class = (item.appid, item.classid, item.instanceid);
                
                if let Some(classinfo) = self.classinfo_cache.get_classinfo(&class) {
                    inventory.push(Asset {
                        classinfo,
                        appid: item.appid,
                        contextid: item.contextid,
                        assetid: item.assetid,
                        amount: item.amount,
                    });
                } else {
                    let instanceid = match item.instanceid {
                        Some(instanceid) => instanceid,
                        None => 0,
                    };
                    
                    return Err(APIError::ResponseError(format!("Missing descriptions for item {}:{}", item.classid, instanceid).into()));
                }
            }
            
            // for body in responses {
            //     for item in &body.assets {
            //         if let Some(classinfo) = body.descriptions.get(&(item.classid, item.instanceid)) {
            //             inventory.push(Asset {
            //                 classinfo: Arc::clone(classinfo),
            //                 appid: item.appid,
            //                 contextid: item.contextid,
            //                 assetid: item.assetid,
            //                 amount: item.amount,
            //             });
            //         } else {
            //             let instanceid = match item.instanceid {
            //                 Some(instanceid) => instanceid,
            //                 None => 0,
            //             };
                        
            //             return Err(APIError::ResponseError(format!("Missing descriptions for item {}:{}", item.classid, instanceid).into()));
            //         }
            //     }
            // }
            
            Ok(inventory)
        }
    }
    
    pub async fn get_inventory(&mut self, steamid: &SteamID, appid: u32, contextid: u32, tradable_only: bool) -> Result<Inventory, APIError> {
        let responses = &mut Vec::new();
        let inventory: Vec<Asset> = self.get_inventory_request(responses, None, steamid, appid, contextid, tradable_only).await?;
        
        Ok(inventory)
    }
}

#[derive(Deserialize, Debug)]
struct GetTradeOffersResponseBody {
    trade_offers_sent: Vec<RawTradeOffer>,
    trade_offers_received: Vec<RawTradeOffer>,
    // #[serde(deserialize_with = "to_classinfo_map")]
    // descriptions: HashMap<(u64, u64), Arc<ClassInfo>>,
    next_cursor: u32,
}

#[derive(Deserialize, Debug)]
struct GetTradeOffersResponse {
    response: GetTradeOffersResponseBody,
}

#[derive(Deserialize, Debug)]
struct RawTradeOffer {
    #[serde(with = "string")]
    tradeofferid: u64,
    accountid_other: u32,
    message: Option<String>,
    #[serde(default)]
    items_to_receive: Vec<RawAsset>,
    #[serde(default)]
    items_to_give: Vec<RawAsset>,
    #[serde(default)]
    is_our_offer: bool,
    #[serde(default)]
    from_real_time_trade: bool,
    #[serde(with = "ts_seconds")]
    expiration_time: ServerTime,
    #[serde(with = "ts_seconds")]
    time_created: ServerTime,
    #[serde(with = "ts_seconds")]
    time_updated: ServerTime,
    trade_offer_state: TradeOfferState,
    // todo parse 0 responses as null
    #[serde(with = "ts_seconds")]
    escrow_end_date: ServerTime,
    confirmation_method: ConfirmationMethod,
}

#[derive(Deserialize, Debug)]
struct RawAsset {
    appid: u32,
    #[serde(with = "string")]
    contextid: u32,
    #[serde(with = "string")]
    assetid: u64,
    #[serde(with = "string")]
    classid: u64,
    #[serde(with = "option_string")]
    instanceid: Option<u64>,
    #[serde(with = "string")]
    amount: u32,
}

#[derive(Deserialize, Debug)]
struct GetInventoryResponse {
    #[serde(default)]
    #[serde(deserialize_with = "from_int_to_bool")]
    success: bool,
    #[serde(default)]
    #[serde(deserialize_with = "from_int_to_bool")]
    more_items: bool,
    #[serde(default)]
    assets: Vec<RawAsset>,
    #[serde(deserialize_with = "to_classinfo_map")]
    descriptions: HashMap<ClassInfoAppClass, Arc<ClassInfo>>,
    #[serde(default)]
    #[serde(deserialize_with = "option_str_to_number")]
    last_assetid: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct GetAssetClassInfoResponse {
    #[serde(deserialize_with = "deserialize_classinfo_map_raw")]
    result: HashMap<ClassInfoAppClass, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;
    use serde::de::DeserializeOwned;

    fn read_file(filename: &str) -> std::io::Result<String> {
        let rootdir = env!("CARGO_MANIFEST_DIR");
        let filepath = Path::new(rootdir).join(format!("tests/json/{}", filename));
        
        fs::read_to_string(filepath)
    }
    
    fn read_and_parse_file<D>(filename: &str) -> Result<D, &str>
    where
        D: DeserializeOwned
    {
        let contents = tests::read_file(filename)
            .expect("Something went wrong reading the file");
        let response: D = serde_json::from_str(&contents).unwrap();
        
        Ok(response)
    }
    
    #[test]
    fn parses_get_asset_classinfo_response() {
        let response: GetAssetClassInfoResponse = tests::read_and_parse_file("get_asset_classinfo.json").unwrap();
        let classinfo_string = response.result.get(&(101785959, Some(11040578))).unwrap();
        let parsed = serde_json::from_str::<ClassInfo>(classinfo_string).unwrap();

        assert_eq!(parsed.market_hash_name, String::from("Mann Co. Supply Crate Key"));
    }
}