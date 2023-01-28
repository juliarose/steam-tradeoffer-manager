use super::{TradeOfferManager, USER_AGENT_STRING};
use crate::{SteamID, ClassInfoCache, helpers::get_default_data_directory};
use std::{path::PathBuf, sync::{Mutex, Arc}};
use reqwest::cookie::Jar;
use reqwest_middleware::ClientWithMiddleware;

/// Builder for constructing a trade offer manager.
pub struct TradeOfferManagerBuilder {
    /// Your account's Steam ID.
    pub steamid: SteamID,
    /// Your account's API key from https://steamcommunity.com/dev/apikey
    pub api_key: String,
    /// The identity secret for the account (optional). Required for mobile confirmations.
    pub identity_secret: Option<String>,
    /// The language for API responses.
    pub language: String,
    /// The [`ClassInfoCache`] to use for this manager. Useful if instantiating multiple managers 
    /// to share state.
    pub classinfo_cache: Arc<Mutex<ClassInfoCache>>,
    /// The location to save data to.
    pub data_directory: PathBuf,
    /// Request cookies.
    pub cookies: Option<Arc<Jar>>,
    /// Client to use for requests. Remember to also include the cookies connected to this client.
    pub client: Option<ClientWithMiddleware>,
    /// User agent for requests.
    pub user_agent: &'static str,
}

impl TradeOfferManagerBuilder {
    /// Creates a new [`TradeOfferManagerBuilder`].
    pub fn new(
        steamid: SteamID,
        api_key: String,
    ) -> Self {
        Self {
            steamid,
            api_key,
            identity_secret: None,
            language: String::from("english"),
            classinfo_cache: Arc::new(Mutex::new(ClassInfoCache::default())),
            data_directory: get_default_data_directory(),
            cookies: None,
            client: None,
            user_agent: USER_AGENT_STRING,
        }
    }
    
    /// The identity secret for the account (optional). Required for mobile confirmations.
    pub fn identity_secret(mut self, identity_secret: String) -> Self {
        self.identity_secret = Some(identity_secret);
        self
    }
    
    /// The language for API responses.
    pub fn language(mut self, language: String) -> Self {
        self.language = language;
        self
    }
    
    /// The [ClassInfoCache] to use for this manager. Useful if instantiation multiple managers 
    /// to share state.
    pub fn classinfo_cache(mut self, classinfo_cache: Arc<Mutex<ClassInfoCache>>) -> Self {
        self.classinfo_cache = classinfo_cache;
        self
    }
    
    /// The location to save data to.
    pub fn data_directory(mut self, data_directory: PathBuf) -> Self {
        self.data_directory = data_directory;
        self
    }
    
    /// Client to use for requests. Remember to also include the cookies connected to this client
    /// or you will need to set the cookies outside of the module.
    pub fn client(mut self, client: ClientWithMiddleware) -> Self {
        self.client = Some(client);
        self
    }
    
    /// Request cookies.
    pub fn cookies(mut self, cookies: Arc<Jar>) -> Self {
        self.cookies = Some(cookies);
        self
    }
    
    /// User agent for requests. If you provided a client this is not needed.
    pub fn user_agent(mut self, user_agent: &'static str) -> Self {
        self.user_agent = user_agent;
        self
    }
    
    /// Builds the [`TradeOfferManager`].
    pub fn build(self) -> TradeOfferManager {
        self.into()
    }
}