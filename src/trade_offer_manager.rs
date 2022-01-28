use std::{
    cmp,
    sync::Arc,
    collections::HashMap
};
use crate::{
    SteamTradeOfferAPI,
    APIError,
    ServerTime,
    OfferFilter,
    TradeOfferState,
    time,
    response,
    request,
    types::{
        AppId,
        ContextId,
        Inventory, TradeOfferId
    }
};
use steamid_ng::SteamID;

#[derive(Debug)]
pub struct TradeOfferManager<'o> {
    api: SteamTradeOfferAPI,
    offers_since: Option<ServerTime>,
    last_poll: Option<ServerTime>,
    last_poll_full_update: Option<ServerTime>,
    poll_data: HashMap<TradeOfferId, Arc<response::TradeOffer<'o>>>,
}

impl<'o> TradeOfferManager<'o> {

    pub fn new(key: String) -> Self {
        Self {
            api: SteamTradeOfferAPI::new(key),
            offers_since: None,
            last_poll: None,
            last_poll_full_update: None,
            poll_data: HashMap::new(),
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

    pub async fn do_poll(&'o mut self, full_update: bool) -> Result<Poll<'o>, APIError> {
        fn last_poll_outdated(last_poll_update: Option<ServerTime>) -> bool {
            match last_poll_update {
                Some(last_poll_full_update) => last_poll_full_update.timestamp()  >= 120000,
                None => true,
            }
        }

        if let Some(last_poll) = self.last_poll {
            let seconds_since_last_poll = time::get_server_time_now().timestamp() - last_poll.timestamp();
                
            if seconds_since_last_poll <= 2 {
                // We last polled less than a second ago... we shouldn't spam the API
                return Err(APIError::ParameterError("Poll called too soon after last poll"))
            }            
        }
        
        let mut offers_since = 0;
        let mut filter = OfferFilter::ActiveOnly;

        self.last_poll = Some(time::get_server_time_now());
        
        if full_update || last_poll_outdated(self.last_poll_full_update) {
            filter = OfferFilter::All;
            offers_since = 1;
            self.last_poll_full_update = Some(time::get_server_time_now())
        } else if let Some(poll_offers_since) = self.offers_since {
            // It looks like sometimes Steam can be dumb and backdate a modified offer. We need to handle this.
            // Let's add a 30-minute buffer.
            offers_since = poll_offers_since.timestamp() + 1800;
        }

        let historical_cutoff = time::timestamp_to_server_time(offers_since as u64);
        let offers = self.api.get_trade_offers(&filter, &Some(historical_cutoff)).await?;
        let mut offers_since: i64 = 0;
        let mut poll = Poll {
            new: Vec::new(),
            changed: Vec::new(),
        };

        for offer in offers {
            offers_since = cmp::max(offers_since, offer.time_updated.timestamp());

            match self.poll_data.get(&offer.tradeofferid) {
                Some(poll_offer) => {
                    if poll_offer.trade_offer_state != offer.trade_offer_state {
                        let offer = Arc::new(offer);

                        poll.changed.push(PollChange {
                            old_state: poll_offer.trade_offer_state.clone(),
                            new_state: offer.trade_offer_state.clone(),
                            offer: Arc::clone(&offer),
                        });
                        
                        self.poll_data.insert(offer.tradeofferid, Arc::clone(&offer));
                    }
                },
                None => {
                    let offer = Arc::new(offer);

                    poll.new.push(Arc::clone(&offer));
                    
                    self.poll_data.insert(offer.tradeofferid, Arc::clone(&offer));
                },
            }
        }

        self.offers_since = Some(time::timestamp_to_server_time(offers_since as u64));

        Ok(poll)
    }
}

pub struct Poll<'o> {
    pub new: Vec<Arc<response::TradeOffer<'o>>>,
    pub changed: Vec<PollChange<'o>>,
}

pub struct PollChange<'o> {
    pub old_state: TradeOfferState,
    pub new_state: TradeOfferState,
    pub offer: Arc<response::TradeOffer<'o>>,
}