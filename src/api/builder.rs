use super::SteamTradeOfferAPI;
use crate::helpers::USER_AGENT_STRING;
use crate::helpers::default_data_directory;
use crate::ClassInfoCache;
use crate::enums::Language;
use std::path::PathBuf;
use std::sync::Arc;
use reqwest::cookie::Jar;
use reqwest_middleware::ClientWithMiddleware;

/// Builder for constructing a [`SteamTradeOfferAPI`].
#[derive(Debug, Clone)]
pub struct SteamTradeOfferAPIBuilder {
    /// Your account's API key from <https://steamcommunity.com/dev/apikey>.
    pub api_key: Option<String>,
    /// The language for API responses.
    pub language: Language,
    /// The [`ClassInfoCache`] to use for this manager. Useful if instantiating multiple managers 
    /// to share state.
    pub classinfo_cache: Option<ClassInfoCache>,
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
    /// Creates a new [`SteamTradeOfferAPIBuilder`].
    pub fn new() -> Self {
        Self {
            api_key: None,
            language: Language::English,
            classinfo_cache: None,
            data_directory: default_data_directory(),
            cookies: None,
            client: None,
            user_agent: USER_AGENT_STRING,
        }
    }
    
    /// The API key. Some features will work without an API key and only require cookies, such as 
    /// sending or responding to trade offers. It is required for all Steam API requests, such 
    /// as getting trade offers or trade histories.
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    
    /// The `data_directory` is the directory used to store poll data and classinfo data.
    pub fn data_directory<T>(mut self, data_directory: T) -> Self
    where
        T: Into<PathBuf>,
    {
        self.data_directory = data_directory.into();
        self
    }
    
    /// The language for API responses.
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }
    
    /// The [`ClassInfoCache`] to use for this manager. Useful if instantiating multiple managers 
    /// to share state.
    pub fn classinfo_cache(mut self, classinfo_cache: ClassInfoCache) -> Self {
        self.classinfo_cache = Some(classinfo_cache);
        self
    }
    
    /// Client to use for requests. It is also required to include the associated cookies with this
    /// client so that the `set_cookies` method works as expected.
    pub fn client(mut self, client: ClientWithMiddleware, cookies: Arc<Jar>) -> Self {
        self.client = Some(client);
        self.cookies = Some(cookies);
        self
    }
    
    /// Builds the [`SteamTradeOfferAPI`].
    pub fn build(self) -> SteamTradeOfferAPI {
        self.into()
    }
}