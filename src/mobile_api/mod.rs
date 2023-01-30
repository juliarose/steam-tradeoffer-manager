// Most of the code here is taken from https://github.com/dyc3/steamguard-cli with some 
// modifications to fit with the rest of this crate.

mod helpers;

use serde::Deserialize;
use reqwest::cookie::Jar;
use url::Url;
use reqwest_middleware::ClientWithMiddleware;
use std::{collections::HashMap, sync::{Arc, RwLock}};
use crate::{SteamID, error::{Error, ParameterError}, helpers::parses_response, response::Confirmation};

/// The API for mobile confirmations.
#[derive(Debug, Clone)]
pub struct MobileAPI {
    pub client: ClientWithMiddleware,
    pub cookies: Arc<Jar>,
    pub language: String,
    pub steamid: SteamID,
    pub identity_secret: Option<String>,
    pub sessionid: Arc<RwLock<Option<String>>>,
}

impl MobileAPI {
    pub const HOSTNAME: &str = "https://steamcommunity.com";
    
    fn get_uri(
        &self,
        pathname: &str,
    ) -> String {
        format!("{}{pathname}", Self::HOSTNAME)
    }
    
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
            .ok_or(ParameterError::NoIdentitySecret)?;
        // let time = self.get_server_time().await?;
        let time = helpers::server_time(0);
        let key = helpers::generate_confirmation_hash_for_time(
            time,
            tag,
            identity_secret,
        )?;
        let mut params: HashMap<&str, String> = HashMap::new();
        
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
