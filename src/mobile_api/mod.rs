// Most of the code here is taken from https://github.com/dyc3/steamguard-cli with some 
// modifications to fit with the rest of this crate.

mod helpers;
mod operation;

use operation::Operation;
use another_steam_totp::{get_device_id, Tag, generate_confirmation_key};
use serde::Deserialize;
use reqwest::cookie::Jar;
use url::Url;
use reqwest_middleware::ClientWithMiddleware;
use std::{collections::HashMap, sync::{Arc, RwLock, atomic::{Ordering, AtomicU64}}};
use crate::{
    SteamID,
    error::{Error, ParameterError},
    helpers::parses_response,
    response::Confirmation,
};

/// The API for mobile confirmations.
#[derive(Debug, Clone)]
pub struct MobileAPI {
    /// The client for making requests.
    pub client: ClientWithMiddleware,
    /// The cookies to make requests with. Since the requests are made with the provided client, 
    /// the cookies should be the same as what the client uses.
    pub cookies: Arc<Jar>,
    /// The language for descriptions.
    pub language: String,
    /// The session ID.
    pub sessionid: Arc<RwLock<Option<String>>>,
    /// The SteamID  of the logged in user. `0` if no login cookies were passed.
    pub steamid: Arc<AtomicU64>,
    /// The identity secret for mobile confirmations.
    pub identity_secret: Option<String>,
    /// The time offset from Steam's servers.
    pub time_offset: i64,
}

impl MobileAPI {
    pub const HOSTNAME: &str = "https://steamcommunity.com";
    
    /// Sets cookies.
    pub fn set_cookies(
        &self,
        cookies: &[String],
    ) {
        let url = Self::HOSTNAME.parse::<Url>()
            .unwrap_or_else(|_| panic!("URL could not be parsed from {}", Self::HOSTNAME));
        
        for cookie_str in cookies {
            self.cookies.add_cookie_str(cookie_str, &url);
        }
    }
    
    /// Sets session.
    pub fn set_session(
        &self,
        sessionid: &str,
        cookies: &[String],
    ) {
        *self.sessionid.write().unwrap() = Some(sessionid.to_string());
        self.set_cookies(cookies);
    }
    
    /// Accepts a confirmation.
    pub async fn accept_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.send_confirmation_ajax(confirmation, Operation::Allow).await
    }

    /// Cancels a confirmation.
    pub async fn cancel_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.send_confirmation_ajax(confirmation, Operation::Cancel).await
    }
    
    /// Gets the trade confirmations.
    pub async fn get_trade_confirmations(
        &self,
    ) -> Result<Vec<Confirmation>, Error> {
        let uri = self.get_uri("/mobileconf/conf");
        let query = self.get_confirmation_query_params(Tag::Conf)?;
        let response = self.client.get(&uri)
            .header("X-Requested-With", "com.valvesoftware.android.steam.community")
            .query(&query)
            .send()
            .await?;
        let body = response.text().await?;
        let confirmations = helpers::parse_confirmations(body)?;
        
        Ok(confirmations)
    }
    
    fn get_confirmation_query_params<'a>(
        &self,
        tag: Tag,
    ) -> Result<HashMap<&'a str, String>, Error> {
        let steamid = self.get_steamid()?;
        let identity_secret = self.identity_secret.as_ref()
            .ok_or(ParameterError::NoIdentitySecret)?;
        let (key, time) = generate_confirmation_key(
            identity_secret.to_owned(),
            tag,
            Some(self.time_offset),
        )?;
        let mut params: HashMap<&str, String> = HashMap::new();
        let device_id = get_device_id(u64::from(steamid));
        
        params.insert("p", device_id);
        params.insert("a", u64::from(steamid).to_string());
        params.insert("k", key);
        params.insert("t", time.to_string());
        params.insert("m", "android".into());
        params.insert("tag", tag.to_string());
        
        Ok(params)
    }
    
    async fn send_confirmation_ajax(
        &self,
        confirmation: &Confirmation,
        operation: Operation,
    ) -> Result<(), Error>  {
        #[derive(Debug, Clone, Copy, Deserialize)]
        struct SendConfirmationResponse {
            pub success: bool,
        }
        
        let mut query = self.get_confirmation_query_params(Tag::Conf)?;
        
        query.insert("op", operation.to_string());
        query.insert("cid", confirmation.id.to_string());
        query.insert("ck", confirmation.key.to_string());
        
        let uri = self.get_uri("/mobileconf/ajaxop");
        let response = self.client.get(&uri)
            .header("X-Requested-With", "com.valvesoftware.android.steam.community")
            .query(&query)
            .send()
            .await?;
        let body: SendConfirmationResponse = parses_response(response).await?;
        
        if !body.success {
            return Err(Error::ConfirmationUnsuccessful);
        }
        
        Ok(())
    }
    
    /// Gets the logged-in user's SteamID.
    fn get_steamid(
        &self,
    ) -> Result<SteamID, Error> {
        let steamid_64 = self.steamid.load(Ordering::Relaxed);
        
        if steamid_64 == 0 {
            return Err(Error::NotLoggedIn);
        }
        
        Ok(SteamID::from(steamid_64))
    }
    
    fn get_uri(
        &self,
        pathname: &str,
    ) -> String {
        format!("{}{pathname}", Self::HOSTNAME)
    }
}
