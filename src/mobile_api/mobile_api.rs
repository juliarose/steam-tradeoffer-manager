use steamid_ng::SteamID;
use serde::Deserialize;
use hmacsha1::hmac_sha1;
use reqwest::cookie::Jar;
use url::{Url, ParseError};
use reqwest_middleware::ClientWithMiddleware;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock}
};
use crate::{
    APIError,
    ParseHtmlError,
    time,
    serializers::string,
    api_helpers::{
        get_default_middleware,
        parses_response
    }
};
use super::{Confirmation, ConfirmationType};
use sha1::{Sha1, Digest};
use lazy_regex::regex_replace_all;
use scraper::{Html, Selector, element_ref::ElementRef};

const HOSTNAME: &'static str = "https://steamcommunity.com";
const API_HOSTNAME: &'static str = "https://api.steampowered.com";
const USER_AGENT_STRING: &'static str = "Mozilla/5.0 (Linux; U; Android 4.1.1; en-us; Google Nexus 4 - 4.1.1 - API 16 - 768x1280 Build/JRO03S) AppleWebKit/534.30 (KHTML, like Gecko) Version/4.0 Mobile Safari/534.30";

fn build_time_bytes(time: i64) -> [u8; 8] {
	time.to_be_bytes()
}
    
fn generate_confirmation_hash_for_time(time: i64, tag: &str, identity_secret: &String) -> String {
    let decode: &[u8] = &base64::decode(&identity_secret).unwrap();
    let time_bytes = build_time_bytes(time);
    let tag_bytes = tag.as_bytes();
    let array = [&time_bytes, tag_bytes].concat();
    let hash = hmac_sha1(decode, &array);
    let encoded = base64::encode(hash);
    
    encoded
}

fn get_device_id(steamid: &SteamID) -> String {
    let mut hasher = Sha1::new();

    hasher.update(u64::from(steamid.clone()).to_string().as_bytes());
    
    let result = hasher.finalize();
    let hash = result.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let device_id = regex_replace_all!(
        r#"^([0-9a-f]{8})([0-9a-f]{4})([0-9a-f]{4})([0-9a-f]{4})([0-9a-f]{12}).*$"#i,
        &hash,
        |_, a, b, c, d, e| format!("{}-{}-{}-{}-{}", a, b, c, d, e),
    );
    
    format!("android:{}", device_id)
}

fn parse_confirmations(text: String) -> Result<Vec<Confirmation>, ParseHtmlError> {
    fn parse_description(element: ElementRef, description_selector: &Selector) -> Result<Confirmation, ParseHtmlError> {
		let description: Option<_> = element.select(&description_selector).next();
        let data_type = element.value().attr("data-type");
        let id = element.value().attr("data-confid");
        let key = element.value().attr("data-key");
        let creator = element.value().attr("data-creator");
        
        // check contents before unwrapping
        if description.is_none() || data_type.is_none() || key.is_none() || creator.is_none() {
            return Err(ParseHtmlError::Malformed("Unexpected description format"));
        }
        
        let description = description
            .unwrap()
            .text()
            .map(|t| t.trim())
            .filter(|t| t.len() > 0)
            .collect::<Vec<_>>()
            .join(" ");
        let conf_type = data_type
            .unwrap()
            .try_into()
            .unwrap_or(ConfirmationType::Unknown);
        
        Ok(Confirmation {
            id: id.unwrap().parse::<u64>()?,
            key: key.unwrap().parse::<u64>()?,
            conf_type,
            description,
            creator: creator.unwrap().parse::<u64>()?,
        })
    }

	let fragment = Html::parse_fragment(&text);
    // these should probably never fail
    let mobileconf_empty_selector = Selector::parse("#mobileconf_empty").unwrap();
    let mobileconf_done_selector = Selector::parse(".mobileconf_done").unwrap();
    let div_selector = Selector::parse("div").unwrap();
    
    if let Some(element) = fragment.select(&mobileconf_empty_selector).next() {
        if mobileconf_done_selector.matches(&element) {
            if let Some(element) = element.select(&div_selector).nth(1) {
                let error_message = element
                    .text()
                    .collect::<String>();
                
                return Err(ParseHtmlError::Response(error_message));
            } else {
                return Ok(Vec::new());
            }
        } else {
            return Ok(Vec::new());
        }
    }
    
	let confirmation_list_selector = Selector::parse(".mobileconf_list_entry").unwrap();
	let description_selector = Selector::parse(".mobileconf_list_entry_description").unwrap();
	let confirmations = fragment.select(&confirmation_list_selector)
        .map(|description| parse_description(description, &description_selector))
        .collect::<Result<Vec<Confirmation>, ParseHtmlError>>()?;
    
    Ok(confirmations)
}

fn server_time(time_offset: i64) -> i64 {
    time::get_system_time() as i64 + time_offset
}

#[derive(Debug)]
pub struct MobileAPI {
    client: ClientWithMiddleware,
    pub cookies: Arc<Jar>,
    pub language: String,
    pub steamid: SteamID,
    pub identity_secret: Option<String>,
    pub sessionid: Arc<RwLock<Option<String>>>,
}

impl MobileAPI {
    
    pub fn new(steamid: &SteamID, identity_secret: Option<String>) -> Self {
        let url = HOSTNAME.parse::<Url>().unwrap();
        let cookies = Arc::new(Jar::default());
        
        cookies.add_cookie_str("mobileClientVersion=0 (2.1.3)", &url);
		cookies.add_cookie_str("mobileClient=android", &url);
		cookies.add_cookie_str("Steam_Language=english", &url);
        cookies.add_cookie_str("dob=", &url);
		cookies.add_cookie_str(format!("steamid={}", u64::from(steamid.clone()).to_string()).as_str(), &url);
        
        Self {
            client: get_default_middleware(Arc::clone(&cookies), USER_AGENT_STRING),
            steamid: steamid.clone(),
            identity_secret,
            language: String::from("english"),
            cookies: Arc::clone(&cookies),
            sessionid: Arc::new(RwLock::new(None)),
        }
    }
    
    fn get_uri(&self, pathname: &str) -> String {
        format!("{}{}", HOSTNAME, pathname)
    }

    fn get_api_url(&self, interface: &str, method: &str, version: usize) -> String {
        format!("{}/{}/{}/v{}", API_HOSTNAME, interface, method, version)
    }
    
    // probably would never fail
    fn set_cookies(&self, cookies: &Vec<String>) -> Result<(), ParseError> {
        let url = HOSTNAME.parse::<Url>()?;
        
        for cookie_str in cookies {
            self.cookies.add_cookie_str(cookie_str, &url);
        }
        
        Ok(())
    }
    
    pub fn set_session(&self, sessionid: &str, cookies: &Vec<String>) -> Result<(), ParseError> {
        let mut sessionid_write = self.sessionid.write().unwrap();
        
        *sessionid_write = Some(sessionid.to_string());
        
        self.set_cookies(cookies)?;
        
        Ok(())
    }
    
	async fn get_confirmation_query_params(&self, tag: &str) -> Result<HashMap<&str, String>, APIError> {
        if self.identity_secret.is_none() {
            return Err(APIError::Parameter("No identity secret"));
        }
        
		// let time = self.get_server_time().await?;
        let time = server_time(0);
        let key = generate_confirmation_hash_for_time(time, tag, &self.identity_secret.clone().unwrap());
		let mut params: HashMap<&str, String> = HashMap::new();
        
        // self.device_id.clone()
		params.insert("p", get_device_id(&self.steamid));
		params.insert("a", u64::from(self.steamid.clone()).to_string());
		params.insert("k", key);
		params.insert("t", time.to_string());
		params.insert("m", "android".into());
		params.insert("tag", tag.into());
		
        Ok(params)
	}
    
	pub async fn send_confirmation_ajax(&self, confirmation: &Confirmation, operation: String) -> Result<(), APIError>  {
		#[derive(Debug, Clone, Copy, Deserialize)]
		struct SendConfirmationResponse {
			pub success: bool,
		}
        
		let mut query = self.get_confirmation_query_params("conf").await?;
        
		query.insert("op", operation);
		query.insert("cid", confirmation.id.to_string());
		query.insert("ck", confirmation.key.to_string());

        let uri = self.get_uri("/mobileconf/ajaxop");
		let response = self.client.get(&uri)
			.header("X-Requested-With", "com.valvesoftware.android.steam.community")
			.query(&query)
			.send()
            .await?;
        // let body: SendConfirmationResponse = parses_response(response).await?;
        let body = response.text().await?;
        
        println!("{}", body);
        
		Ok(())
	}

	pub async fn accept_confirmation(&self, confirmation: &Confirmation) -> Result<(), APIError> {
		self.send_confirmation_ajax(confirmation, "allow".into()).await
	}

	pub async fn deny_confirmation(&self, confirmation: &Confirmation) -> Result<(), APIError> {
		self.send_confirmation_ajax(confirmation, "cancel".into()).await
	}
    
    pub async fn get_server_time(&self) -> Result<i64, APIError> {
        #[derive(Deserialize, Debug)]
        struct ServerTime {
            #[serde(with = "string")]
            server_time: i64,
            // skew_tolerance_seconds: u32,
            // large_time_jink: u32,
            // probe_frequency_seconds: u32,
            // adjusted_time_probe_frequency_seconds: u32,
            // hint_probe_frequency_seconds: u32,
            // sync_timeout: u32,
            // try_again_seconds: u32,
            // max_attempts: u32,
        }
        
        #[derive(Deserialize, Debug)]
        struct Response {
            response: ServerTime,
        }
        
        let uri = self.get_api_url("ITwoFactorService", "QueryTime", 1);
        let response = self.client.post(&uri)
            .body("steamid=0")
            .send()
            .await?;
        let body: Response = parses_response(response).await?;
        
        Ok(body.response.server_time)
    }
    
    pub async fn get_trade_confirmations(&self) -> Result<Vec<Confirmation>, APIError> {
        let uri = self.get_uri("/mobileconf/conf");
        let query = self.get_confirmation_query_params("conf").await?;
		let response = self.client.get(&uri)
			.header("X-Requested-With", "com.valvesoftware.android.steam.community")
			.query(&query)
            .send()
            .await?;
		let body = response.text().await?;
        let confirmations = parse_confirmations(body)?;
        
        Ok(confirmations)
    }
}