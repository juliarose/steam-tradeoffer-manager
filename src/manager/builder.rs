use super::TradeOfferManager;
use crate::helpers::USER_AGENT_STRING;
use crate::ClassInfoCache;
use crate::enums::Language;
use std::path::PathBuf;
use std::sync::Arc;
use reqwest::cookie::Jar;
use reqwest_middleware::ClientWithMiddleware;
use directories::BaseDirs;

/// Builder for constructing a [`TradeOfferManager`].
#[derive(Debug, Clone)]
pub struct TradeOfferManagerBuilder {
    /// Your account's API key from <https://steamcommunity.com/dev/apikey>.
    pub api_key: String,
    /// The identity secret for the account (optional). Required for mobile confirmations.
    pub identity_secret: Option<String>,
    /// The language for API responses.
    pub language: Language,
    /// The [`ClassInfoCache`] to use for this manager. Useful if instantiating multiple managers 
    /// to share state.
    pub classinfo_cache: ClassInfoCache,
    /// The location to save data to.
    pub data_directory: PathBuf,
    /// Request cookies.
    pub cookie_jar: Option<Arc<Jar>>,
    /// Client to use for requests. Remember to also include the cookies connected to this client.
    pub client: Option<ClientWithMiddleware>,
    /// User agent for requests.
    pub user_agent: &'static str,
    /// How many seconds your computer is behind Steam's servers. Used in mobile confirmations.
    pub time_offset: i64,
    /// Cookies to set on initialization.
    pub cookies: Option<Vec<String>>,
}

impl TradeOfferManagerBuilder {
    /// Creates a new [`TradeOfferManagerBuilder`].
    /// 
    /// An API key is required to use the Steam Web API. You can get one from the  
    /// [Steam Community](https://steamcommunity.com/dev/apikey) or by using the 
    /// [`TradeOfferManager::get_api_key`][`crate::TradeOfferManager`] method.
    /// 
    /// By default, the data directory is stored in the config directory of the current user 
    /// determined by the OS:
    /// - Linux: `/home/<username>/.config/rust-steam-tradeoffer-manager`
    /// - MacOS: `/Users/<username>/Library/Application Support/rust-steam-tradeoffer-manager`
    /// - Windows: `C:\Users\<username>\AppData\Roaming\rust-steam-tradeoffer-manager`
    /// 
    /// In some cases (such as when running in a Docker container), the config directory may not be 
    /// available. In this case, the data directory will be stored in the 
    /// `rust-steam-tradeoffer-manager` directory in the current working directory.
    /// 
    /// Refer to the [directories](https://docs.rs/directories/5.0.1/directories/struct.BaseDirs.html) crate for more 
    /// information.
    pub fn new(api_key: String) -> Self {
        let data_directory = if let Some(base_dirs) = BaseDirs::new() {
            let config_dir = base_dirs.config_dir().join("rust-steam-tradeoffer-manager");
            
            if !config_dir.exists() {
                std::fs::create_dir_all(&config_dir).ok();
            }
            
            config_dir
        } else {
            "./rust-steam-tradeoffer-manager".into()
        };
        
        Self {
            api_key,
            identity_secret: None,
            language: Language::English,
            classinfo_cache: ClassInfoCache::default(),
            data_directory,
            cookie_jar: None,
            client: None,
            user_agent: USER_AGENT_STRING,
            time_offset: 0,
            cookies: None,
        }
    }
    
    /// The `data_directory` is the directory used to store poll data and classinfo data.
    pub fn data_directory<T>(mut self, data_directory: T) -> Self
    where
        T: Into<PathBuf>,
    {
        self.data_directory = data_directory.into();
        self
    }
    
    /// The identity secret for the account. Required for mobile confirmations.
    pub fn identity_secret(mut self, identity_secret: String) -> Self {
        self.identity_secret = Some(identity_secret);
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
        self.classinfo_cache = classinfo_cache;
        self
    }
    
    /// Client to use for requests. It is also required to include the associated cookies with this
    /// client so that the `set_cookies` method works as expected.
    pub fn client(mut self, client: ClientWithMiddleware, cookie_jar: Arc<Jar>) -> Self {
        self.client = Some(client);
        self.cookie_jar = Some(cookie_jar);
        self
    }
    
    /// How many seconds your computer is behind Steam's servers. Used in mobile confirmations.
    pub fn time_offset(mut self, time_offset: i64) -> Self {
        self.time_offset = time_offset;
        self
    }
    
    /// The API key.
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = api_key;
        self
    }
    
    /// The web cookies.
    pub fn cookies(mut self, cookies: Vec<String>) -> Self {
        self.cookies = Some(cookies);
        self
    }
    
    /// Builds the [`TradeOfferManager`].
    pub fn build(self) -> TradeOfferManager {
        self.into()
    }
}