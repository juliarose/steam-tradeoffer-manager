use super::TradeOfferManager;
use crate::{SteamID, ClassInfoCache};
use std::sync::{RwLock, Arc};

pub struct TradeOfferManagerBuilder {
    pub steamid: SteamID,
    pub key: String,
    pub identity_secret: Option<String>,
    pub language: String,
    pub classinfo_cache: Arc<RwLock<ClassInfoCache>>,
}

impl TradeOfferManagerBuilder {
    
    pub fn new(steamid: SteamID, key: String) -> Self {
        Self {
            steamid,
            key,
            identity_secret: None,
            language: String::from("english"),
            classinfo_cache: Arc::new(RwLock::new(ClassInfoCache::new())),
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
    
    pub fn classinfo_cache(mut self, classinfo_cache: Arc<RwLock<ClassInfoCache>>) -> Self {
        self.classinfo_cache = classinfo_cache;
        self
    }
    
    pub fn build(self) -> TradeOfferManager {
        TradeOfferManager::from(self)
    }
}