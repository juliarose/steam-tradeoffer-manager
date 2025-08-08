use super::{SteamTradeOfferAPI, DEFAULT_GET_INVENTORY_PAGE_SIZE};
use crate::helpers::{Session, default_data_directory, USER_AGENT_STRING};
use crate::ClassInfoCache;
use crate::enums::Language;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use reqwest::cookie::Jar;
use reqwest_middleware::ClientWithMiddleware;

/// Builder for constructing a [`SteamTradeOfferAPI`].
/// 
/// An access token or API key is required to use the Steam Web API. If you provide an API key,
/// requests will prefer using your access token when available. Do not supply an API key if
/// you only want to use access tokens for API requests.
#[derive(Debug, Clone)]
pub struct SteamTradeOfferAPIBuilder {
    /// Your account's API key from <https://steamcommunity.com/dev/apikey>.
    pub(crate) api_key: Option<String>,
    /// The access token for your account.
    pub(crate) access_token: Option<String>,
    /// The language for API responses.
    pub(crate) language: Language,
    /// The number of items to fetch per page when getting inventories. Defaults to 2000.
    pub(crate) get_inventory_page_size: u32,
    /// The [`ClassInfoCache`] to use for this manager. Useful if instantiating multiple managers
    /// to share state.
    pub(crate) classinfo_cache: Option<ClassInfoCache>,
    /// The location to save data to.
    pub(crate) data_directory: PathBuf,
    /// Request cookies.
    pub(crate) cookie_jar: Option<Arc<Jar>>,
    /// Client to use for requests. Remember to also include the cookies connected to this client.
    pub(crate) client: Option<ClientWithMiddleware>,
    /// User agent for requests.
    pub(crate) user_agent: &'static str,
    /// The session.
    pub(crate) session: Option<Arc<RwLock<Option<Session>>>>,
}

impl Default for SteamTradeOfferAPIBuilder {
    fn default() -> Self {
        Self {
            api_key: None,
            access_token: None,
            language: Language::English,
            get_inventory_page_size: DEFAULT_GET_INVENTORY_PAGE_SIZE,
            classinfo_cache: None,
            data_directory: default_data_directory(),
            cookie_jar: None,
            client: None,
            user_agent: USER_AGENT_STRING,
            session: None,
        }
    }
}

impl SteamTradeOfferAPIBuilder {
    /// Creates a new [`SteamTradeOfferAPIBuilder`].
    pub fn new() -> Self {
        Self::default()
    }
    
    /// The API key. Some features will work without an API key and only require cookies, such as
    /// sending or responding to trade offers. It is required for all Steam API requests, such
    /// as getting trade offers or trade histories.
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    
    /// The access token. Some features will work without an access token and only require cookies,
    /// such as sending or responding to trade offers. It is required for all Steam API requests, 
    /// such as getting trade offers or trade histories.
    pub fn access_token(mut self, access_token: String) -> Self {
        self.access_token = Some(access_token);
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
    
    /// The number of items to fetch per page when getting inventories. Defaults to 2000.
    pub fn get_inventory_page_size(mut self, page_size: u32) -> Self {
        self.get_inventory_page_size = page_size;
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
        self.cookie_jar = Some(cookies);
        self
    }
    
    /// Sets the session.
    pub(crate) fn session(mut self, session: Arc<RwLock<Option<Session>>>) -> Self {
        self.session = Some(session);
        self
    }
    
    /// Builds the [`SteamTradeOfferAPI`].
    pub fn build(self) -> SteamTradeOfferAPI {
        self.into()
    }
}
