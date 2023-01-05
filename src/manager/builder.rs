use super::{file, PollData, TradeOfferManager, USER_AGENT_STRING};
use crate::{
    SteamID,
    api::SteamTradeOfferAPI,
    mobile_api::MobileAPI,
    ClassInfoCache,
    helpers::{get_default_middleware, get_default_data_directory},
};
use std::{path::PathBuf, sync::{Mutex, RwLock, Arc}};
use chrono::Duration;
use reqwest::cookie::Jar;
use reqwest_middleware::ClientWithMiddleware;

/// Builder for constring a trade offer manager.
pub struct TradeOfferManagerBuilder {
    /// Your account's Steam ID.
    pub steamid: SteamID,
    /// Your account's API key from https://steamcommunity.com/dev/apikey
    pub key: String,
    /// The identity secret for the account (optional). Required for mobile confirmations.
    pub identity_secret: Option<String>,
    /// The language for API responses.
    pub language: String,
    /// The [ClassInfoCache] to use for this manager. Useful if instantiation multiple managers 
    /// to share state.
    pub classinfo_cache: Arc<Mutex<ClassInfoCache>>,
    /// The duration after a sent offer has been active to cancel during a poll. Offers will 
    /// not be cancelled if this is not set.
    pub cancel_duration: Option<Duration>,
    /// The location to save data to.
    pub data_directory: PathBuf,
    /// Request cookies.
    pub cookies: Option<Arc<Jar>>,
    /// Client to use for requests.
    pub client: Option<ClientWithMiddleware>,
    /// User agent for requests.
    pub user_agent: &'static str,
}

impl TradeOfferManagerBuilder {
    pub fn new(
        steamid: SteamID,
        key: String,
    ) -> Self {
        Self {
            steamid,
            key,
            identity_secret: None,
            language: String::from("english"),
            classinfo_cache: Arc::new(Mutex::new(ClassInfoCache::default())),
            cancel_duration: None,
            data_directory: get_default_data_directory(),
            cookies: None,
            client: None,
            user_agent: USER_AGENT_STRING,
        }
    }
    
    pub fn identity_secret(mut self, identity_secret: String) -> Self {
        self.identity_secret = Some(identity_secret);
        self
    }

    pub fn language(mut self, language: String) -> Self {
        self.language = language;
        self
    }
    
    pub fn classinfo_cache(mut self, classinfo_cache: Arc<Mutex<ClassInfoCache>>) -> Self {
        self.classinfo_cache = classinfo_cache;
        self
    }
    
    pub fn cancel_duration(mut self, duration: Duration) -> Self {
        self.cancel_duration = Some(duration);
        self
    }
    
    pub fn data_directory(mut self, data_directory: PathBuf) -> Self {
        self.data_directory = data_directory;
        self
    }
    
    pub fn client(mut self, client: ClientWithMiddleware) -> Self {
        self.client = Some(client);
        self
    }
    
    pub fn cookies(mut self, cookies: Arc<Jar>) -> Self {
        self.cookies = Some(cookies);
        self
    }
    
    pub fn user_agent(mut self, user_agent: &'static str) -> Self {
        self.user_agent = user_agent;
        self
    }
    
    pub fn build(self) -> TradeOfferManager {
        let cookies = self.cookies.unwrap_or_else(|| Arc::new(Jar::default()));
        let client = self.client.unwrap_or_else(|| {
            get_default_middleware(
                Arc::clone(&cookies),
                self.user_agent,
            )
        });
        let steamid = self.steamid;
        let identity_secret = self.identity_secret;
        let poll_data = file::load_poll_data(
            &steamid,
            &self.data_directory,
        ).unwrap_or_else(|_| PollData::new());
        let language = self.language;
        let mobile_api_client = client.clone();
        
        TradeOfferManager {
            steamid: self.steamid,
            api: SteamTradeOfferAPI::new(
                Arc::clone(&cookies),
                client,
                steamid,
                self.key,
                language.clone(),
                identity_secret.clone(),
                self.classinfo_cache,
                self.data_directory.clone(),
            ),
            mobile_api: MobileAPI::new(
                cookies,
                mobile_api_client,
                steamid,
                language,
                identity_secret,
            ),
            poll_data: Arc::new(RwLock::new(poll_data)),
            cancel_duration: self.cancel_duration,
            data_directory: self.data_directory,
        }
    }
}