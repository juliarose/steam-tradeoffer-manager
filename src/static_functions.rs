//! Exported functions in lib.rs

use crate::api::response as api_response;
use crate::response::{Asset, ClassInfo};
use crate::request::GetInventoryOptions;
use crate::types::*;
use crate::helpers::{parses_response, extract_auth_data_from_cookies};
use crate::helpers::COMMUNITY_HOSTNAME;
use crate::error::{Error, ParseHtmlError, MissingClassInfoError};
use crate::serialize;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use reqwest::cookie::Jar;
use reqwest::header::REFERER;
use scraper::{Html, Selector};
use url::Url;

const ERROR_COULD_NOT_GET_API_KEY: &str = "API key could not be parsed from response";
const ERROR_NO_API_KEY: &str = "This account does not have an API key";

/// A stand-alone method for getting a user's inventory. Optionally allows specifying a client to
/// use for requests (useful if you need to proxy your requests, for example).
/// 
/// # Examples
/// ```no_run
/// use steam_tradeoffer_manager::get_inventory;
/// use steam_tradeoffer_manager::request::GetInventoryOptions;
/// use steamid_ng::SteamID;
/// 
/// #[tokio::main]
/// async fn main() {
///     let options = GetInventoryOptions {
///         steamid: SteamID::from(76561199436464454),
///         appid: 730,
///         contextid: 2,
///         tradable_only: false,
///         ..Default::default()
///     };
///     let inventory = get_inventory(&options).await.unwrap();
///     
///     println!("{} item(s) in CS:GO inventory", inventory.len());
/// }
/// ```
pub async fn get_inventory<'a>(
    options: &GetInventoryOptions<'a>,
) -> Result<Vec<Asset>, Error> { 
    #[derive(Serialize)]
    struct Query<'a> {
        l: &'a str,
        count: u32,
        start_assetid: Option<u64>,
    }
    
    let mut responses: Vec<GetInventoryResponse> = Vec::new();
    let mut start_assetid: Option<u64> = None;
    let sid = u64::from(options.steamid);
    let appid = options.appid;
    let contextid = options.contextid;
    let uri = format!("https://{COMMUNITY_HOSTNAME}/inventory/{sid}/{appid}/{contextid}");
    let referer = format!("https://{COMMUNITY_HOSTNAME}/profiles/{sid}/inventory");
    
    loop {
        let response = options.client.get(&uri)
            .header(REFERER, &referer)
            .query(&Query {
                l: options.language.api_language_code(),
                count: options.page_size,
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
    
    for body in responses {
        let mut items = body.assets
            .iter()
            .filter_map(|item| {
                let classinfo_result = body.descriptions
                    .get(&(item.classid, item.instanceid))
                    .ok_or_else(|| Error::MissingClassInfo(MissingClassInfoError {
                        appid,
                        classid: item.classid,
                        instanceid: item.instanceid,
                    }));
                
                match classinfo_result {
                    Ok(classinfo) if options.tradable_only && !classinfo.tradable => {
                        None
                    },
                    Ok(classinfo) => Some(Ok(Asset {
                        appid,
                        contextid,
                        assetid: item.assetid,
                        amount: item.amount,
                        missing: false,
                        classinfo: Arc::clone(classinfo),
                    })),
                    Err(error) => Some(Err(error)),
                }
            })
            .collect::<Result<_, _>>()?;
        
        inventory.append(&mut items);
    }
    
    Ok(inventory)
}

/// Gets your Steam Web API key.
/// 
/// This method requires your cookies. If your account does not have an API key set, one will be
/// created using `localhost` as the domain. By calling this method you are agreeing to the
/// [Steam Web API Terms of Use](https://steamcommunity.com/dev/apiterms).
pub async fn get_api_key(
    cookies: &[String],
) -> Result<String, Error> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CreateAPIKey {
        domain: String,
        agree_to_terms: String,
        sessionid: String,
        submit: String,
    }
    
    let (
        sessionid,
        _steamid,
        _access_token,
    ) = extract_auth_data_from_cookies(cookies);
    let sessionid = sessionid
        .ok_or(Error::NotLoggedIn)?;
    let cookie_store = Arc::new(Jar::default());
    // Should not panic since the URL is hardcoded.
    let url = format!("https://{COMMUNITY_HOSTNAME}").parse::<Url>()
        .unwrap_or_else(|error| panic!("URL could not be parsed from {COMMUNITY_HOSTNAME}: {error}"));
    
    for cookie in cookies {
        cookie_store.add_cookie_str(cookie, &url);
    }
    
    let client = reqwest::ClientBuilder::new()
        .cookie_provider(cookie_store)
        .build()?;
    
    match try_get_key(&client).await {
        Ok(api_key) => Ok(api_key),
        Err(Error::ParseHtml(ParseHtmlError::Malformed(message))) if message == ERROR_NO_API_KEY => {
            let uri = format!("https://{COMMUNITY_HOSTNAME}/dev/registerkey");
            let _response = client.post(uri)
                .form(&CreateAPIKey {
                    domain: "localhost".into(),
                    sessionid,
                    agree_to_terms: "agreed".into(),
                    submit: "Register".into(),
                })
                .send()
                .await?;
            
            try_get_key(&client).await
        },
        Err(error) => Err(error),
    }
}

/// Makes a request to https://steamcommunity.com/dev/apikey and scrapes the page for the API key.
async fn try_get_key(client: &reqwest::Client) -> Result<String, Error> {
    let uri = format!("https://{COMMUNITY_HOSTNAME}/dev/apikey");
    let response = client.get(uri)
        .send()
        .await?;
    let text = response.text().await?;
    let fragment = Html::parse_fragment(&text);
    let main_selector = Selector::parse("#mainContents h2")
        .map_err(|_error| ParseHtmlError::ParseSelector)?;
    let body_contents_selector = Selector::parse("#bodyContents_ex")
        .map_err(|_error| ParseHtmlError::ParseSelector)?;
    let h2_selector = Selector::parse("h2")
        .map_err(|_error| ParseHtmlError::ParseSelector)?;
    let p_selector = Selector::parse("p")
        .map_err(|_error| ParseHtmlError::ParseSelector)?;
    
    if let Some(element) = fragment.select(&main_selector).next() {
        if element.text().collect::<String>() == "Access Denied" {
            return Err(Error::NotLoggedIn);
        }
    }
    
    if let Some(body_contents_element) = fragment.select(&body_contents_selector).next() {
        if let Some(element) = body_contents_element.select(&h2_selector).next() {
            if element.text().collect::<String>().trim() == "Your Steam Web API Key" {
                if let Some(element) = body_contents_element.select(&p_selector).next() {
                    let text = element.text().collect::<String>();
                    let mut text = text.trim().split(' ');
                    
                    text.next();
                    
                    if let Some(api_key) = text.next() {
                        return Ok(api_key.to_string());
                    } else {
                        return Err(ParseHtmlError::Malformed(ERROR_COULD_NOT_GET_API_KEY).into());
                    }
                }
            }
        }
    }
    
    Err(ParseHtmlError::Malformed(ERROR_NO_API_KEY).into())
}

#[derive(Deserialize)]
struct GetInventoryResponse {
    #[serde(default)]
    #[serde(deserialize_with = "serialize::from_int_to_bool")]
    success: bool,
    #[serde(default)]
    #[serde(deserialize_with = "serialize::from_int_to_bool")]
    more_items: bool,
    #[serde(default)]
    assets: Vec<api_response::RawAsset>,
    #[serde(default)]
    #[serde(deserialize_with = "serialize::to_classinfo_map")]
    descriptions: HashMap<ClassInfoAppClass, Arc<ClassInfo>>,
    #[serde(default)]
    #[serde(deserialize_with = "serialize::option_str_to_number")]
    last_assetid: Option<u64>,
}

#[test]
fn parses_get_inventory_response() {
    let response: GetInventoryResponse = serde_json::from_str(include_str!("api/fixtures/inventory.json")).unwrap();
    let asset = response.assets.first().unwrap();
    
    assert_eq!(asset.assetid, 11152148507);
}
