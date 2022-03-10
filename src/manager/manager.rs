use std::{
    cmp,
    sync::{Arc, RwLock},
    collections::HashMap
};
use crate::{
    APIError,
    ServerTime,
    OfferFilter,
    TradeOfferState,
    time,
    response,
    request,
    api::SteamTradeOfferAPI,
    error::FileError,
    mobile_api::{MobileAPI, Confirmation},
    types::{
        AppId,
        ContextId,
        TradeOfferId
    }
};
use steamid_ng::SteamID;
use url::ParseError;
use super::{
    Poll,
    PollChange,
    file,
    poll_data::PollData
};

#[derive(Debug)]
pub struct TradeOfferManager {
    steamid: SteamID,
    // manager facades api
    api: SteamTradeOfferAPI,
    mobile_api: MobileAPI,
    poll_data: Arc<RwLock<PollData>>,
}

impl PollData {
    
    pub fn new() -> Self {
        Self {
            offers_since: None,
            last_poll: None,
            last_poll_full_update: None,
            state_map: HashMap::new(),
        }
    }
}

impl TradeOfferManager {

    pub fn new(
        steamid: &SteamID,
        key: &str,
        identity_secret: Option<String>,
    ) -> Self {
        Self {
            steamid: steamid.clone(),
            api: SteamTradeOfferAPI::new(steamid, key, identity_secret.clone()),
            mobile_api: MobileAPI::new(steamid, identity_secret),
            poll_data: Arc::new(RwLock::new(PollData::new())),
        }
    }
    
    pub fn new_with_poll_data(
        steamid: &SteamID,
        key: &str,
        identity_secret: Option<String>,
    ) -> Self {
        let poll_data = file::load_poll_data(steamid).unwrap_or_else(|_| PollData::new());
        
        Self {
            steamid: steamid.clone(),
            api: SteamTradeOfferAPI::new(steamid, key, identity_secret.clone()),
            mobile_api: MobileAPI::new(steamid, identity_secret),
            poll_data: Arc::new(RwLock::new(poll_data)),
        }
    }
    
    pub fn set_session(
        &self,
        sessionid: &str,
        cookies: &Vec<String>,
    ) -> Result<(), ParseError> {
        self.api.set_session(sessionid, cookies)?;
        self.mobile_api.set_session(sessionid, cookies)?;
        
        Ok(())
    }

    pub fn create_offer(
        &self,
        partner: &SteamID,
        message: Option<String>,
        token: Option<String>,
    ) -> request::new_trade_offer::NewTradeOffer {
        request::new_trade_offer::NewTradeOffer {
            id: None,
            partner: partner.clone(),
            token,
            message,
            items_to_give: Vec::new(),
            items_to_receive: Vec::new(),
        }
    }
    
    pub async fn send_offer(
        &self,
        offer: &request::new_trade_offer::NewTradeOffer,
    ) -> Result<response::sent_offer::SentOffer, APIError> {
        self.api.send_offer(offer).await
    }
    
    pub async fn accept_offer(
        &self,
        offer: &response::trade_offer::TradeOffer,
    ) -> Result<response::accepted_offer::AcceptedOffer, APIError> {
        if offer.is_our_offer {
            return Err(APIError::Parameter("Cannot accept an offer that is ours"));
        } else if offer.trade_offer_state != TradeOfferState::Active {
            return Err(APIError::Parameter("Cannot accept an offer that is not active"));
        }

        self.api.accept_offer(offer.tradeofferid, &offer.partner).await
    }
    
    pub async fn cancel_offer(
        &self,
        offer: &response::trade_offer::TradeOffer,
    ) -> Result<(), APIError> {
        if !offer.is_our_offer {
            return Err(APIError::Parameter("Cannot cancel an offer we did not create"));
        }
        
        self.api.cancel_offer(offer.tradeofferid).await
    }
    
    pub async fn decline_offer(
        &self,
        offer: &response::trade_offer::TradeOffer,
    ) -> Result<(), APIError> {
        if offer.is_our_offer {
            return Err(APIError::Parameter("Cannot decline an offer we created"));
        }
        
        self.api.decline_offer(offer.tradeofferid).await
    }

    pub async fn get_trade_offers(
        &self
    ) -> Result<Vec<response::trade_offer::TradeOffer>, APIError> {
        self.api.get_trade_offers(&OfferFilter::ActiveOnly, &None).await
    }

    pub async fn get_inventory_old(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, APIError> {
        self.api.get_inventory_old(steamid, appid, contextid, tradable_only).await
    }
    
    pub async fn get_inventory(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, APIError> {
        self.api.get_inventory(steamid, appid, contextid, tradable_only).await
    }
    
    async fn save_poll_data(&self) -> Result<(), FileError> {
        // we clone this so we don't hold it across an await
        let poll_data = self.poll_data.read().unwrap().clone();
        let data = serde_json::to_string(&poll_data)?;
        
        file::save_poll_data(&self.steamid, &data).await
    }
    
    pub async fn get_user_details(
        &self,
        tradeofferid: &Option<TradeOfferId>,
        partner: &SteamID,
        token: &Option<String>,
    ) -> Result<response::user_details::UserDetails, APIError> {
        self.api.get_user_details(tradeofferid, partner, token).await
    }
    
    pub async fn get_trade_confirmations(
        &self,
    ) -> Result<Vec<Confirmation>, APIError> {
        self.mobile_api.get_trade_confirmations().await
    }
    
    pub async fn accept_confirmation(
        &self,
        confirmaton: &Confirmation,
    ) -> Result<(), APIError> {
        self.mobile_api.accept_confirmation(confirmaton).await
    }
    
    pub async fn decline_confirmation(
        &self,
        confirmaton: &Confirmation,
    ) -> Result<(), APIError> {
        self.mobile_api.deny_confirmation(confirmaton).await
    }
    
    pub async fn get_receipt(&self, offer: &response::trade_offer::TradeOffer) -> Result<Vec<response::asset::Asset>, APIError> {
        if offer.items_to_receive.is_empty() {
            Ok(Vec::new())
        } else if let Some(tradeid) = offer.tradeid {
            self.api.get_receipt(&tradeid).await
        } else {
            Err(APIError::Parameter("Missing tradeid".into()))
        }
    }
    
    pub async fn update_offer(&self, offer: &mut response::trade_offer::TradeOffer) -> Result<(), APIError> {
        let updated = self.api.get_trade_offer(offer.tradeofferid).await?;
        
        offer.tradeofferid = updated.tradeofferid;
        offer.tradeid = updated.tradeid;
        offer.trade_offer_state = updated.trade_offer_state;
        offer.confirmation_method = updated.confirmation_method;
        offer.escrow_end_date = updated.escrow_end_date;
        offer.time_created = updated.time_created;
        offer.time_updated = updated.time_updated;
        offer.expiration_time = updated.expiration_time;
        
        Ok(())
    }

    pub async fn do_poll(
        &self,
        full_update: bool,
    ) -> Result<Poll, APIError> {
        fn date_difference_from_now(date: &ServerTime) -> i64 {
            let current_timestamp = time::get_server_time_now().timestamp();
            
            current_timestamp - date.timestamp()
        }
        
        fn last_poll_full_outdated(last_poll_full_update: Option<ServerTime>) -> bool {
            match last_poll_full_update {
                Some(last_poll_full_update) => {
                    date_difference_from_now(&last_poll_full_update) >= 120
                },
                None => true,
            }
        }
        
        let mut offers_since = 0;
        let mut filter = OfferFilter::ActiveOnly;
        
        {
            let mut poll_data = self.poll_data.write().unwrap();

            if let Some(last_poll) = poll_data.last_poll {
                let seconds_since_last_poll = date_difference_from_now(&last_poll);
                    
                if seconds_since_last_poll <= 1 {
                    // We last polled less than a second ago... we shouldn't spam the API
                    return Err(APIError::Parameter("Poll called too soon after last poll"));
                }            
            }
            
            poll_data.last_poll = Some(time::get_server_time_now());
        
            if full_update || last_poll_full_outdated(poll_data.last_poll_full_update) {
                filter = OfferFilter::All;
                offers_since = 1;
                poll_data.last_poll_full_update = Some(time::get_server_time_now())
            } else if let Some(poll_offers_since) = poll_data.offers_since {
                // It looks like sometimes Steam can be dumb and backdate a modified offer. We need to handle this.
                // Let's add a 30-minute buffer.
                offers_since = poll_offers_since.timestamp() + 1800;
            }
        }

        let historical_cutoff = time::timestamp_to_server_time(offers_since);
        let offers = self.api.get_trade_offers(&filter, &Some(historical_cutoff)).await?;
        let mut offers_since: i64 = 0;
        let mut poll: Poll = Vec::new();
        
        {
            let mut poll_data = self.poll_data.write().unwrap();
                
            for offer in offers {
                offers_since = cmp::max(offers_since, offer.time_updated.timestamp());

                match poll_data.state_map.get(&offer.tradeofferid) {
                    Some(poll_trade_offer_state) => {
                        if poll_trade_offer_state != &offer.trade_offer_state {
                            let tradeofferid = offer.tradeofferid;
                            let new_state = offer.trade_offer_state.clone();
                            
                            poll.push((offer, Some(poll_trade_offer_state.clone())));
                            
                            poll_data.state_map.insert(tradeofferid, new_state);
                        }
                    },
                    None => {
                        poll_data.state_map.insert(offer.tradeofferid, offer.trade_offer_state.clone());
                        
                        poll.push((offer, None));
                    },
                }
            }
            
            if offers_since > 0 {
                poll_data.offers_since = Some(time::timestamp_to_server_time(offers_since));
            }
            
        }
        
        let _ = self.save_poll_data().await;
        
        Ok(poll)
    }
}