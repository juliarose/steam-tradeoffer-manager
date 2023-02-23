use super::SteamTradeOfferAPI;
use crate::helpers::USER_AGENT_STRING;
use crate::ClassInfoCache;
use crate::enums::Language;
use std::path::PathBuf;
use std::sync::{Mutex, Arc};
use reqwest::cookie::Jar;
use reqwest_middleware::ClientWithMiddleware;

/// Builder for constructing a [`SteamTradeOfferAPI`].
pub struct SteamTradeOfferAPIBuilder {
    /// Your account's API key from <https://steamcommunity.com/dev/apikey>.
    pub api_key: String,
    /// The language for API responses.
    pub language: Language,
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

impl SteamTradeOfferAPIBuilder {
    /// Creates a new [`SteamTradeOfferAPIBuilder`]. The `data_directory` is the directory used to 
    /// store poll data and classinfo data.
    pub fn new<T>(
        api_key: String,
        data_directory: T,
    ) -> Self
    where
        T: Into<PathBuf>,
    {
        Self {
            api_key,
            language: Language::English,
            classinfo_cache: Arc::new(Mutex::new(ClassInfoCache::default())),
            data_directory: data_directory.into(),
            cookies: None,
            client: None,
            user_agent: USER_AGENT_STRING,
        }
    }
    
    /// The language for API responses.
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }
    
    /// The [`ClassInfoCache`] to use for this manager. Useful if instantiating multiple managers.
    /// to share state.
    pub fn classinfo_cache(mut self, classinfo_cache: Arc<Mutex<ClassInfoCache>>) -> Self {
        self.classinfo_cache = classinfo_cache;
        self
    }
    
    /// Client to use for requests. It is also required to include the associated cookies with this
    /// client so that the `set_cookies` method works as expected.
    pub fn client(mut self, client: ClientWithMiddleware, cookies: Arc<Jar>) -> Self {
        self.client = Some(client);
        self.cookies = Some(cookies);
        self
    }
    
    /// The API key.
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = api_key;
        self
    }
    
    /// Builds the [`SteamTradeOfferAPI`].
    pub fn build(self) -> SteamTradeOfferAPI {
        self.into()
    }
}