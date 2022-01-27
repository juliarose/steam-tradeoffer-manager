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

    pub fn create_offer(&self, partner: SteamID, token: Option<String>, message: Option<String>) -> request::NewTradeOffer {
        request::NewTradeOffer {
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

    pub async fn do_poll<'a>(&'a mut self, full_update: bool) -> Result<Vec<response::TradeOffer<'a>>, APIError> {
        fn last_poll_outdated(last_poll_update: Option<ServerTime>) -> bool {
            match last_poll_update {
                Some(last_poll_full_update) => last_poll_full_update.timestamp()  >= 120000,
                None => true,
            }
        }
        
        let mut offers_since: u64 = 0;
        let mut filter = OfferFilter::ActiveOnly;

        self.last_poll = Some(time::get_server_time_now());
        
        if full_update || last_poll_outdated(self.last_poll_full_update) {
            filter = OfferFilter::All;
            offers_since = 1;
            self.last_poll_full_update = Some(time::get_server_time_now())
        }
        
        let historical_cutoff = time::timestamp_to_server_time(offers_since);

        self.api.get_trade_offers(&filter, &Some(historical_cutoff)).await
    }
}