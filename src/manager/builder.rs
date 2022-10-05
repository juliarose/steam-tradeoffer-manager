use super::TradeOfferManager;
use crate::{SteamID, ClassInfoCache};
use std::sync::{Mutex, Arc};
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
    /// A 
    pub classinfo_cache: Arc<Mutex<ClassInfoCache>>,
    pub cancel_duration: Option<Duration>,
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
    
    pub fn build(self) -> TradeOfferManager {
        TradeOfferManager::from(self)
    }
}