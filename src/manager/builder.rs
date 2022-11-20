use super::TradeOfferManager;
use crate::{SteamID, ClassInfoCache, helpers::get_default_data_directory};
use std::{path::PathBuf, sync::{Mutex, Arc}};
use chrono::Duration;

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
    
    pub fn build(self) -> TradeOfferManager {
        TradeOfferManager::from(self)
    }
}