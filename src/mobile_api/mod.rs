// Most of the code here is taken from https://github.com/dyc3/steamguard-cli with some 
// modifications to fit with the rest of this crate.

mod confirmation;
mod helpers;

pub use confirmation::{Confirmation, ConfirmationType};

use serde::Deserialize;
use reqwest::cookie::Jar;
use url::Url;
use reqwest_middleware::ClientWithMiddleware;
use std::{collections::HashMap, sync::{Arc, RwLock}};
use crate::{SteamID, error::Error, helpers::parses_response};

#[derive(Debug, Clone)]
pub struct MobileAPI {
    client: ClientWithMiddleware,
    pub cookies: Arc<Jar>,
    pub language: String,
    pub steamid: SteamID,
    pub identity_secret: Option<String>,
    pub sessionid: Arc<RwLock<Option<String>>>,
}

impl MobileAPI {
    pub const HOSTNAME: &str = "https://steamcommunity.com";
    
    /// Creates a new [`MobileAPI`].
    pub fn new(
        cookies: Arc<Jar>,
        client: ClientWithMiddleware,
        steamid: SteamID,
        language: String,
        identity_secret: Option<String>,
    ) -> Self {
        // I would only hope this never fails...
        let url = Self::HOSTNAME.parse::<Url>().unwrap();
        
        cookies.add_cookie_str("mobileClientVersion=0 (2.1.3)", &url);
        cookies.add_cookie_str("mobileClient=android", &url);
        cookies.add_cookie_str("Steam_Language=english", &url);
        cookies.add_cookie_str("dob=", &url);
        cookies.add_cookie_str(format!("steamid={}", u64::from(steamid)).as_str(), &url);
        
        Self {
            client,
            steamid,
            identity_secret,
            language,
            cookies,
            sessionid: Arc::new(RwLock::new(None)),
        }
    }
    
    fn get_uri(
        &self,
        pathname: &str,
    ) -> String {
        format!("{}{}", Self::HOSTNAME, pathname)
    }
    
    /// Sets cookies.
    pub fn set_cookies(
        &self,
        cookies: &Vec<String>,
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
        cookies: &Vec<String>,
    ) {
        *self.sessionid.write().unwrap() = Some(sessionid.to_string());
        self.set_cookies(cookies);
    }
    
    /// Accepts a confirmation.
    pub async fn accept_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.send_confirmation_ajax(confirmation, "allow").await
    }

    /// Cancels a confirmation.
    pub async fn cancel_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.send_confirmation_ajax(confirmation, "cancel").await
    }
    
    /// Gets the trade confirmations.
    pub async fn get_trade_confirmations(
        &self,
    ) -> Result<Vec<Confirmation>, Error> {
        let uri = self.get_uri("/mobileconf/conf");
        let query = self.get_confirmation_query_params("conf")?;
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
        tag: &str,
    ) -> Result<HashMap<&'a str, String>, Error> {
        let identity_secret = self.identity_secret.as_ref()
            .ok_or_else(|| Error::Parameter("No identity secret"))?;
        // let time = self.get_server_time().await?;
        let time = helpers::server_time(0);
        let key = helpers::generate_confirmation_hash_for_time(
            time,
            tag,
            identity_secret,
        )?;
        let mut params: HashMap<&str, String> = HashMap::new();
        
        // self.device_id.clone()
        params.insert("p", helpers::get_device_id(&self.steamid));
        params.insert("a", u64::from(self.steamid).to_string());
        params.insert("k", key);
        params.insert("t", time.to_string());
        params.insert("m", "android".into());
        params.insert("tag", tag.into());
        
        Ok(params)
    }
    
    async fn send_confirmation_ajax(
        &self,
        confirmation: &Confirmation,
        operation: &str,
    ) -> Result<(), Error>  {
        #[derive(Debug, Clone, Copy, Deserialize)]
        struct SendConfirmationResponse {
            pub success: bool,
        }
        
        let mut query = self.get_confirmation_query_params("conf")?;
        
        query.insert("op", operation.into());
        query.insert("cid", confirmation.id.to_string());
        query.insert("ck", confirmation.key.to_string());
        
        let uri = self.get_uri("/mobileconf/ajaxop");
        let response = self.client.get(&uri)
            .header("X-Requested-With", "com.valvesoftware.android.steam.community")
            .query(&query)
            .send()
            .await?;
        // let body: SendConfirmationResponse = parses_response(response).await?;
        let body: SendConfirmationResponse = parses_response(response).await?;
        
        if !body.success {
            return Err(Error::ConfirmationUnsuccessful);
        }
        
        Ok(())
    }
}
