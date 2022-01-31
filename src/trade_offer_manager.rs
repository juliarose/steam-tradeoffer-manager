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
pub struct TradeOfferManager {
    // manager facades api
    api: SteamTradeOfferAPI,
    offers_since: Option<ServerTime>,
    last_poll: Option<ServerTime>,
    last_poll_full_update: Option<ServerTime>,
    poll_data: HashMap<TradeOfferId, TradeOfferState>,
}

impl TradeOfferManager {

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
            id: None,
            partner,
            token,
            message,
            items_to_give: Vec::new(),
            items_to_receive: Vec::new(),
        }
    }
    
    pub async fn send_offer(&self, offer: &request::NewTradeOffer) -> Result<response::SentOffer, APIError> {
        self.api.send_offer(offer).await
    }
    
    pub async fn accept_offer(&self, offer: &response::TradeOffer) -> Result<response::AcceptedOffer, APIError> {
        if offer.is_our_offer {
            return Err(APIError::ParameterError("Cannot accept an offer that is ours"));
        } else if offer.trade_offer_state != TradeOfferState::Active {
            return Err(APIError::ParameterError("Cannot accept an offer that is not active"));
        }

        self.api.accept_offer(offer.tradeofferid, &offer.partner).await
    }
    
    pub async fn cancel_offer(&self, offer: &response::TradeOffer) -> Result<(), APIError> {
        if !offer.is_our_offer {
            return Err(APIError::ParameterError("Cannot cancel an offer we did not create"));
        }
        
        self.api.cancel_offer(offer.tradeofferid).await
    }
    
    pub async fn decline_offer(&self, offer: &response::TradeOffer) -> Result<(), APIError> {
        if offer.is_our_offer {
            return Err(APIError::ParameterError("Cannot decline an offer we created"));
        }
        
        self.api.decline_offer(offer.tradeofferid).await
    }

    pub async fn get_trade_offers(&mut self) -> Result<Vec<response::TradeOffer>, APIError> {
        self.api.get_trade_offers(&OfferFilter::ActiveOnly, &None).await
    }

    pub async fn get_inventory_old(&mut self, steamid: &SteamID, appid: AppId, contextid: ContextId, tradable_only: bool) -> Result<Inventory, APIError> {
        self.api.get_inventory_old(steamid, appid, contextid, tradable_only).await
    }
    
    pub async fn get_inventory(&mut self, steamid: &SteamID, appid: AppId, contextid: ContextId, tradable_only: bool) -> Result<Inventory, APIError> {
        self.api.get_inventory(steamid, appid, contextid, tradable_only).await
    }

    pub async fn do_poll(&mut self, full_update: bool) -> Result<Poll, APIError> {
        fn date_difference_from_now(date: &ServerTime) -> i64 {
            let current_timestamp = time::get_server_time_now().timestamp();
            
            current_timestamp - date.timestamp()
        }
        
        fn last_poll_full_outdated(last_poll_full_update: Option<ServerTime>) -> bool {
            match last_poll_full_update {
                Some(last_poll_full_update) => {
                    println!("{}", date_difference_from_now(&last_poll_full_update));
                    date_difference_from_now(&last_poll_full_update) >= 120
                },
                None => true,
            }
        }

        if let Some(last_poll) = self.last_poll {
            let seconds_since_last_poll = date_difference_from_now(&last_poll);
                
            if seconds_since_last_poll <= 2 {
                // We last polled less than a second ago... we shouldn't spam the API
                return Err(APIError::ParameterError("Poll called too soon after last poll"));
            }            
        }
        
        let mut offers_since = 0;
        let mut filter = OfferFilter::ActiveOnly;

        self.last_poll = Some(time::get_server_time_now());
        
        if full_update || last_poll_full_outdated(self.last_poll_full_update) {
            filter = OfferFilter::All;
            offers_since = 1;
            self.last_poll_full_update = Some(time::get_server_time_now())
        } else if let Some(poll_offers_since) = self.offers_since {
            // It looks like sometimes Steam can be dumb and backdate a modified offer. We need to handle this.
            // Let's add a 30-minute buffer.
            offers_since = poll_offers_since.timestamp() + 1800;
        }

        println!("{:?}", offers_since);
        let historical_cutoff = time::timestamp_to_server_time(offers_since);
        
        println!("{:?}", self.last_poll_full_update);
        println!("{:?}", historical_cutoff);
        let offers = self.api.get_trade_offers(&filter, &Some(historical_cutoff)).await?;
        let mut offers_since: i64 = 0;
        let mut poll = Poll {
            new: Vec::new(),
            changed: Vec::new(),
        };
        // println!("{:?}", offers);
        
        for offer in offers {
            offers_since = cmp::max(offers_since, offer.time_updated.timestamp());

            match self.poll_data.get(&offer.tradeofferid) {
                Some(poll_trade_offer_state) => {
                    if poll_trade_offer_state != &offer.trade_offer_state {
                        let tradeofferid = offer.tradeofferid;
                        let new_state = offer.trade_offer_state.clone();
                        
                        poll.changed.push(PollChange {
                            old_state: poll_trade_offer_state.clone(),
                            new_state: offer.trade_offer_state.clone(),
                            offer,
                        });
                        
                        self.poll_data.insert(tradeofferid, new_state);
                    }
                },
                None => {
                    self.poll_data.insert(offer.tradeofferid, offer.trade_offer_state.clone());
                    
                    poll.new.push(offer);
                },
            }
        }
        
        if offers_since > 0 {
            self.offers_since = Some(time::timestamp_to_server_time(offers_since));
        }

        Ok(poll)
    }
}

#[derive(Debug)]
pub struct Poll {
    pub new: Vec<response::TradeOffer>,
    pub changed: Vec<PollChange>,
}

#[derive(Debug)]
pub struct PollChange {
    pub old_state: TradeOfferState,
    pub new_state: TradeOfferState,
    pub offer: response::TradeOffer,
}