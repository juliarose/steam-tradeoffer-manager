use std::time::SystemTime;
use crate::{
    SteamTradeOfferAPI,
    APIError,
    ServerTime,
    OfferFilter,
    time,
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
    last_poll: Option<ServerTime>,
    last_poll_full_update: Option<ServerTime>,
}

impl TradeOfferManager {

    pub fn new(key: String) -> Self {
        Self {
            api: SteamTradeOfferAPI::new(key),
            last_poll: None,
            last_poll_full_update: None,
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
        self.api.get_trade_offers(&OfferFilter::ActiveOnly, &None).await
    }

    pub async fn get_inventory_old(&mut self, steamid: &SteamID, appid: AppId, contextid: ContextId, tradable_only: bool) -> Result<Inventory, APIError> {
        self.api.get_inventory_old(steamid, appid, contextid, tradable_only).await
    }
    
    pub async fn get_inventory(&mut self, steamid: &SteamID, appid: AppId, contextid: ContextId, tradable_only: bool) -> Result<Inventory, APIError> {
        self.api.get_inventory(steamid, appid, contextid, tradable_only).await
    }

    pub async fn do_poll<'a>(&'a mut self, mut full_update: bool) -> Result<Vec<response::TradeOffer<'a>>, APIError> {
        let mut offers_since: u64 = 0;

        self.last_poll = Some(time::get_server_time_now());

        if let Some(last_poll_full_update) = self.last_poll_full_update {
            if last_poll_full_update.timestamp() >= 120000 {
                full_update = true;
                offers_since = 1;
                self.last_poll_full_update = Some(time::get_server_time_now())
            }
        } else if full_update {
            full_update = true;
            offers_since = 1;
            self.last_poll_full_update = Some(time::get_server_time_now())
        }

        let filter = match full_update {
            true => OfferFilter::All,
            false => OfferFilter::ActiveOnly,
        };
        let historical_cutoff = time::timestamp_to_server_time(offers_since);

        self.api.get_trade_offers(&filter, &Some(historical_cutoff)).await
    }
}