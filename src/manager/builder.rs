use super::TradeOfferManager;
use crate::SteamID;

pub struct TradeOfferManagerBuilder {
    steamid: SteamID,
    key: String,
    identity_secret: Option<String>,
}

impl TradeOfferManagerBuilder {
    
    pub fn new(steamid: &SteamID, key: &str) -> Self {
        Self {
            steamid: *steamid,
            key: key.into(),
            identity_secret: None,
        }
    }

    pub fn identity_secret(mut self, identity_secret: &str) -> Self {
        self.identity_secret = Some(identity_secret.into());
        self
    }
    
    pub fn build(self) -> TradeOfferManager {
        TradeOfferManager::new(&self.steamid, &self.key, self.identity_secret)
    }
}