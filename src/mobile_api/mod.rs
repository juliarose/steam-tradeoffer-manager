//! This is the underlying mobile API for the manager. In most cases you should stick to using the 
//! manager, but if you need more control over the requests, you can use this API directly.

// Most of the code here is taken from https://github.com/dyc3/steamguard-cli with some 
// modifications to fit with the rest of this crate.

mod builder;
mod operation;

use operation::Operation;

pub use builder::MobileAPIBuilder;

use crate::SteamID;
use crate::response::Confirmation;
use crate::error::{Error, ParameterError};
use crate::helpers::{
    parses_response,
    generate_sessionid,
    extract_auth_data_from_cookies,
    get_default_client,
};
use crate::helpers::COMMUNITY_HOSTNAME;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{Ordering, AtomicU64};
use another_steam_totp::{Tag, get_device_id, generate_confirmation_key};
use serde::Deserialize;
use reqwest::cookie::Jar;
use url::Url;
use reqwest_middleware::ClientWithMiddleware;

/// The API for mobile confirmations.
#[derive(Debug, Clone)]
pub struct MobileAPI {
    /// The identity secret for mobile confirmations.
    pub identity_secret: Option<String>,
    /// The time offset from Steam's servers.
    pub time_offset: i64,
    /// The client for making requests.
    client: ClientWithMiddleware,
    /// The cookies to make requests with. Since the requests are made with the provided client, 
    /// the cookies should be the same as what the client uses.
    cookies: Arc<Jar>,
    /// The session ID.
    sessionid: Arc<RwLock<Option<String>>>,
    /// The SteamID  of the logged in user. `0` if no login cookies were passed.
    steamid: Arc<AtomicU64>,
}

impl MobileAPI {
    /// Hostname for requests.
    const HOSTNAME: &'static str = COMMUNITY_HOSTNAME;
    
    /// Builder for constructing a [`MobileAPI`].
    pub fn builder() -> MobileAPIBuilder {
        MobileAPIBuilder::new()
    }
    
    /// Sets cookies.
    /// 
    /// All requests require your cookies to be set. Make sure your cookies are set before using 
    /// this API.
    pub fn set_cookies(
        &self,
        mut cookies: Vec<String>,
    ) {
        let (
            sessionid,
            steamid,
            _access_token,
        ) = extract_auth_data_from_cookies(&cookies);
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
        
        *self.sessionid.write().unwrap() = Some(sessionid);
        
        if let Some(steamid) = steamid {
            self.steamid.store(steamid, Ordering::Relaxed);
        }
        
        for cookie_str in &cookies {
            self.cookies.add_cookie_str(cookie_str, &url);
        }
    }
    
    /// Accepts a confirmation.
    pub async fn accept_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.send_confirmation_ajax(&confirmation.id, &confirmation.nonce, Operation::Allow).await
    }

    /// Cancels a confirmation.
    pub async fn cancel_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.send_confirmation_ajax(&confirmation.id, &confirmation.nonce, Operation::Cancel).await
    }
    
    /// Accepts a confirmation by ID.
    pub async fn accept_confirmation_by_id(
        &self,
        id: u64,
        nonce: u64,
    ) -> Result<(), Error> {
        self.send_confirmation_ajax(&id, &nonce, Operation::Allow).await
    }
    
    /// Cancels a confirmation by ID.
    pub async fn cancel_confirmation_by_id(
        &self,
        id: u64,
        nonce: u64,
    ) -> Result<(), Error> {
        self.send_confirmation_ajax(&id, &nonce, Operation::Cancel).await
    }
    
    /// Gets the trade confirmations.
    pub async fn get_trade_confirmations(
        &self,
    ) -> Result<Vec<Confirmation>, Error> {
        #[derive(Deserialize)]
        pub struct GetTradeConfirmationsResponse {
            // #[serde(default)]
            // pub success: bool,
            #[serde(default)]
            pub conf: Vec<Confirmation>,
        }
        
        let uri = Self::get_url("/mobileconf/getlist");
        let query = self.get_confirmation_query_params(Tag::Conf)?;
        let response = self.client.get(&uri)
            .header("X-Requested-With", "com.valvesoftware.android.steam.community")
            .query(&query)
            .send()
            .await?;
        let response: GetTradeConfirmationsResponse = parses_response(response).await?;
        
        Ok(response.conf)
    }
    
    fn get_confirmation_query_params<'a>(
        &self,
        tag: Tag,
    ) -> Result<HashMap<&'a str, String>, Error> {
        let steamid = self.get_steamid()?;
        let identity_secret = self.identity_secret.as_ref()
            .ok_or(ParameterError::NoIdentitySecret)?;
        let (key, time) = generate_confirmation_key(
            identity_secret,
            tag,
            Some(self.time_offset),
        )?;
        let mut params: HashMap<&str, String> = HashMap::new();
        let device_id = get_device_id(u64::from(steamid));
        
        params.insert("p", device_id);
        params.insert("a", u64::from(steamid).to_string());
        params.insert("k", key);
        params.insert("t", time.to_string());
        params.insert("m", "react".into());
        params.insert("tag", tag.to_string());
        
        Ok(params)
    }
    
    async fn send_confirmation_ajax(
        &self,
        id: &u64,
        nonce: &u64,
        operation: Operation,
    ) -> Result<(), Error>  {
        #[derive(Deserialize)]
        struct SendConfirmationResponse {
            pub success: bool,
            #[serde(default)]
            pub message: Option<String>,
        }
        
        let mut query = self.get_confirmation_query_params(Tag::Conf)?;
        
        query.insert("op", operation.to_string());
        query.insert("cid", id.to_string());
        query.insert("ck", nonce.to_string());
        
        let uri = Self::get_url("/mobileconf/ajaxop");
        let response = self.client.get(&uri)
            .header("X-Requested-With", "com.valvesoftware.android.steam.community")
            .query(&query)
            .send()
            .await?;
        let body: SendConfirmationResponse = parses_response(response).await?;
        
        if !body.success {
            return Err(Error::ConfirmationUnsuccessful(body.message));
        }
        
        Ok(())
    }
    
    /// Gets the logged-in user's SteamID.
    pub fn get_steamid(
        &self,
    ) -> Result<SteamID, Error> {
        let steamid_64 = self.steamid.load(Ordering::Relaxed);
        
        if steamid_64 == 0 {
            return Err(Error::NotLoggedIn);
        }
        
        Ok(SteamID::from(steamid_64))
    }
    
    fn get_url(
        pathname: &str,
    ) -> String {
        format!("https://{}{pathname}", Self::HOSTNAME)
    }
}

impl From<MobileAPIBuilder> for MobileAPI {
    fn from(builder: MobileAPIBuilder) -> Self {
        let cookies = builder.cookies
            .unwrap_or_else(|| Arc::new(Jar::default()));
        let client = builder.client
            .unwrap_or_else(|| get_default_client(
                Arc::clone(&cookies),
                builder.user_agent,
            ));
        
        Self {
            client,
            cookies: Arc::clone(&cookies),
            sessionid: Arc::new(std::sync::RwLock::new(None)),
            identity_secret: builder.identity_secret,
            steamid: Arc::new(AtomicU64::new(0)),
            time_offset: builder.time_offset,
        }
    }
}
