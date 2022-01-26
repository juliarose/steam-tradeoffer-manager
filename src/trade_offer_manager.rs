use reqwest::cookie::CookieStore;

use crate::SteamTradeOfferAPI;

#[derive(Debug)]
pub struct TradeOfferManager {
    api: SteamTradeOfferAPI,
}

impl TradeOfferManager {

    fn new(key: String) -> Self {
        Self {
            api: SteamTradeOfferAPI::new(key)
        }
    }

    pub fn set_cookies(&mut self, cookies: &Vec<String>) {
        self.api.set_cookies(cookies);
    }
}