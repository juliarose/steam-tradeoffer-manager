use crate::{
    SteamTradeOfferAPI,
    APIError,
    response,
    request,
    types::{
        AppId,
        ContextId,
        Inventory
    }
};
use steamid_ng::SteamID;

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

    pub fn create_offer(&self, partner: SteamID, token: Option<String>, message: Option<String>) -> request::CreateTradeOffer {
        request::CreateTradeOffer {
            api: &self.api,
            id: None,
            partner,
            token,
            message,
            items_to_give: Vec::new(),
            items_to_receive: Vec::new(),
        }
    }

    pub async fn get_trade_offers<'a>(&'a mut self) -> Result<Vec<response::TradeOffer<'a>>, APIError> {
        self.api.get_trade_offers().await
    }

    pub async fn get_inventory_old(&mut self, steamid: &SteamID, appid: AppId, contextid: ContextId, tradable_only: bool) -> Result<Inventory, APIError> {
        self.api.get_inventory_old(steamid, appid, contextid, tradable_only).await
    }
    
    pub async fn get_inventory(&mut self, steamid: &SteamID, appid: AppId, contextid: ContextId, tradable_only: bool) -> Result<Inventory, APIError> {
        self.api.get_inventory(steamid, appid, contextid, tradable_only).await
    }
}