use crate::{
    APIError,
    Item,
    Currency,
    response::{
        self,
        ClassInfo,
        deserializers::{
            from_int_to_bool,
            to_classinfo_map
        }
    },
    request,
    api_helpers::{
        get_default_middleware,
        parses_response
    }
};
use async_recursion::async_recursion;
use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_qs;
use reqwest::{cookie::Jar, Url};
use reqwest_middleware::ClientWithMiddleware;
use reqwest::header::REFERER;
use steamid_ng::SteamID;
use std::sync::Arc;
use lazy_regex::regex_captures;
use async_std::task::sleep;
use crate::serializers::{string, option_str_to_number, steamid_as_string};
use std::rc::Rc;

const HOSTNAME: &'static str = "https://steamcommunity.com";

pub struct SteamTradeOfferAPI {
    key: String,
    cookies: Arc<Jar>,
    client: ClientWithMiddleware,
    sessionid: Option<String>,
}

impl SteamTradeOfferAPI {
    
    pub fn new(key: String) -> Self {
        let cookies = Arc::new(Jar::default());
        
        Self {
            key,
            cookies: Arc::clone(&cookies),
            client: get_default_middleware(Arc::clone(&cookies)),
            sessionid: None,
        }
    }
    
    fn get_uri(&self, pathname: &str) -> String {
        format!("{}{}", HOSTNAME, pathname)
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
    
    #[async_recursion]
    async fn get_inventory_request(&self, responses: &mut Vec<GetInventoryResponse>, start_assetid: Option<u64>, steamid: &SteamID, appid: u32, contextid: u32, tradable_only: bool) -> Result<Inventory, APIError> { 
        #[derive(Serialize, Debug)]
        struct Query<'a> {
            l: &'a str,
            count: u32,
            start_assetid: Option<u64>,
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
            
            for body in responses {
                for item in &body.assets {
                    if let Some(classinfo) = body.descriptions.get(&(item.classid, item.instanceid)) {
                        inventory.push(Asset {
                            classinfo: Arc::clone(classinfo),
                            appid: item.appid,
                            contextid: item.contextid,
                            assetid: item.assetid,
                            amount: item.amount,
                        });
                    } else {
                        return Err(APIError::ResponseError(format!("Missing descriptions for item {}:{}", item.classid, item.instanceid).into()));
                    }
                }
            }
            
            Ok(inventory)
        }
    }
    
    pub async fn get_inventory(&self, steamid: &SteamID, appid: u32, contextid: u32, tradable_only: bool) -> Result<Inventory, APIError> {
        let responses = &mut Vec::new();
        let inventory: Vec<Asset> = self.get_inventory_request(responses, None, steamid, appid, contextid, tradable_only).await?;
        
        Ok(inventory)
    }
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
    #[serde(with = "string")]
    instanceid: u64,
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
    descriptions: HashMap<(u64, u64), Arc<ClassInfo>>,
    #[serde(default)]
    #[serde(deserialize_with = "option_str_to_number")]
    last_assetid: Option<u64>,
}

pub type Inventory = Vec<Asset>;

#[derive(Debug)]
pub struct Asset {
    pub appid: u32,
    pub contextid: u32,
    pub assetid: u64,
    pub amount: u32,
    pub classinfo: Arc<ClassInfo>,
}