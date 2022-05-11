mod poll;
mod poll_data;
mod file;
mod builder;

pub use builder::TradeOfferManagerBuilder;
pub use poll::Poll;

use poll_data::PollData;
use std::{cmp, sync::{Arc, RwLock}};
use chrono::Duration;
use crate::{
    error::Error,
    ServerTime,
    enums::{OfferFilter, TradeOfferState},
    time,
    response,
    request,
    api::SteamTradeOfferAPI,
    error::FileError,
    mobile_api::{MobileAPI, Confirmation},
    types::{
        AppId,
        ContextId,
        TradeOfferId,
    },
};
use steamid_ng::SteamID;
use url::ParseError;
use reqwest::cookie::Jar;

/// Manager which includes functionality for interacting with trade offers, confirmations and 
/// inventories.
#[derive(Debug)]
pub struct TradeOfferManager {
    steamid: SteamID,
    // manager facades api
    api: SteamTradeOfferAPI,
    mobile_api: MobileAPI,
    poll_data: Arc<RwLock<PollData>>,
    cancel_duration: Option<Duration>,
}

impl From<TradeOfferManagerBuilder> for TradeOfferManager {
    fn from(builder: TradeOfferManagerBuilder) -> Self {
        let cookies = Arc::new(Jar::default());
        let steamid = builder.steamid;
        let identity_secret = builder.identity_secret;
        let poll_data = file::load_poll_data(&steamid).unwrap_or_else(|_| PollData::new());
        let language = builder.language;
        
        Self {
            steamid,
            api: SteamTradeOfferAPI::new(
                Arc::clone(&cookies),
                steamid,
                builder.key,
                language.clone(),
                identity_secret.clone(),
                builder.classinfo_cache,
            ),
            mobile_api: MobileAPI::new(
                cookies,
                steamid,
                language,
                identity_secret,
            ),
            poll_data: Arc::new(RwLock::new(poll_data)),
            cancel_duration: builder.cancel_duration,
        }
    }
}

impl TradeOfferManager {
    /// Builder for new manager.
    pub fn builder(
        steamid: SteamID,
        key: String,
    ) -> TradeOfferManagerBuilder {
        TradeOfferManagerBuilder::new(steamid, key)
    }
    
    /// Sets the session and cookies.
    pub fn set_session(
        &self,
        sessionid: &str,
        cookies: &Vec<String>,
    ) -> Result<(), ParseError> {
        self.api.set_session(sessionid, cookies)?;
        self.mobile_api.set_session(sessionid, cookies)?;
        
        Ok(())
    }
    
    /// Counters an existing offer.
    pub async fn counter_offer(
        &self,
        offer: &mut response::trade_offer::TradeOffer,
        counter_offer: &request::trade_offer::NewTradeOffer,
    ) -> Result<response::sent_offer::SentOffer, Error> {
        let sent_offer = self.api.send_offer(
            counter_offer,
            Some(offer.tradeofferid),
        ).await?;
        
        offer.trade_offer_state = TradeOfferState::Countered;
        
        Ok(sent_offer)
    }
    
    /// Sends an offer.
    pub async fn send_offer(
        &self,
        offer: &request::trade_offer::NewTradeOffer,
    ) -> Result<response::sent_offer::SentOffer, Error> {
        self.api.send_offer(offer, None).await
    }
    
    /// Accepts an offer.
    pub async fn accept_offer(
        &self,
        offer: &mut response::trade_offer::TradeOffer,
    ) -> Result<response::accepted_offer::AcceptedOffer, Error> {
        if offer.is_our_offer {
            return Err(Error::Parameter("Cannot accept an offer that is ours"));
        } else if offer.trade_offer_state != TradeOfferState::Active {
            return Err(Error::Parameter("Cannot accept an offer that is not active"));
        }

        let accepted_offer = self.api.accept_offer(offer.tradeofferid, &offer.partner).await?;
        offer.trade_offer_state = TradeOfferState::Accepted;
        
        Ok(accepted_offer)
    }
    
    /// Cancels an offer.
    pub async fn cancel_offer(
        &self,
        offer: &mut response::trade_offer::TradeOffer,
    ) -> Result<(), Error> {
        if !offer.is_our_offer {
            return Err(Error::Parameter("Cannot cancel an offer we did not create"));
        }
        
        self.api.cancel_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Canceled;
        
        Ok(())
    }
    
    /// Declines an offer.
    pub async fn decline_offer(
        &self,
        offer: &mut response::trade_offer::TradeOffer,
    ) -> Result<(), Error> {
        if offer.is_our_offer {
            return Err(Error::Parameter("Cannot decline an offer we created"));
        }
        
        self.api.decline_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Declined;
        
        Ok(())
    }

    /// Gets a user's inventory using the old endpoint.
    pub async fn get_inventory_old(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        self.api.get_inventory_old(steamid, appid, contextid, tradable_only).await
    }
    
    /// Gets a user's inventory.
    pub async fn get_inventory(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::asset::Asset>, Error> {
        self.api.get_inventory(steamid, appid, contextid, tradable_only).await
    }
    
    /// Gets the user's details for trading.
    pub async fn get_user_details(
        &self,
        tradeofferid: &Option<TradeOfferId>,
        partner: &SteamID,
        token: &Option<String>,
    ) -> Result<response::user_details::UserDetails, Error> {
        self.api.get_user_details(tradeofferid, partner, token).await
    }
    
    /// Gets trade confirmations.
    pub async fn get_trade_confirmations(
        &self,
    ) -> Result<Vec<Confirmation>, Error> {
        self.mobile_api.get_trade_confirmations().await
    }
    
    /// Confirms a trade offer.
    pub async fn confirm_offer(
        &self,
        trade_offer: &response::TradeOffer,
    ) -> Result<(), Error> {
        self.confirm_offerid(trade_offer.tradeofferid).await
    }
    
    /// Confirms an trade offer using its ID.
    pub async fn confirm_offerid(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<(), Error> {
        let confirmations = self.get_trade_confirmations().await?;
        let confirmation = confirmations
            .into_iter()
            .find(|confirmation| confirmation.creator == tradeofferid);
        
        if let Some(confirmation) = confirmation {
            self.accept_confirmation(&confirmation).await
        } else {
            Err(Error::NoConfirmationForOffer(tradeofferid))
        }
    }
    
    /// Accepts a confirmation.
    pub async fn accept_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.mobile_api.accept_confirmation(confirmation).await
    }
    
    /// Accepts confirmations.
    pub async fn accept_confirmations(
        &self,
        confirmations: &[Confirmation],
    ) -> Result<(), Error> {
        for confirmation in confirmations {
            self.mobile_api.accept_confirmation(confirmation).await?
        }
        
        Ok(())
    }
    
    /// Declines a confirmation.
    pub async fn decline_confirmation(
        &self,
        confirmation: &Confirmation,
    ) -> Result<(), Error> {
        self.mobile_api.deny_confirmation(confirmation).await
    }
    
    /// Gets the trade receipt (new items) upon completion of a trade.
    pub async fn get_receipt(&self, offer: &response::trade_offer::TradeOffer) -> Result<Vec<response::asset::Asset>, Error> {
        if offer.trade_offer_state != TradeOfferState::Accepted {
            Err(Error::Parameter(r#"Offer is not in "accepted" state"#))
        } else if offer.items_to_receive.is_empty() {
            Ok(Vec::new())
        } else if let Some(tradeid) = offer.tradeid {
            self.api.get_receipt(&tradeid).await
        } else {
            Err(Error::Parameter("Missing tradeid"))
        }
    }
    
    /// Updates the offer to the most recent state against the API.
    pub async fn update_offer(&self, offer: &mut response::trade_offer::TradeOffer) -> Result<(), Error> {
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

    /// Gets active trade offers.
    pub async fn get_active_trade_offers(
        &self
    ) -> Result<Vec<response::trade_offer::TradeOffer>, Error> {
        self.api.get_trade_offers(&OfferFilter::ActiveOnly, &None).await
    }
    
    /// Gets trade offers.
    pub async fn get_trade_offers(
        &self,
        filter: &OfferFilter,
        historical_cutoff: &Option<ServerTime>,
    ) -> Result<Vec<response::trade_offer::TradeOffer>, Error> {
        self.api.get_trade_offers(filter, historical_cutoff).await
    }
    
    /// Performs a poll for changes to offers.
    pub async fn do_poll(
        &self,
        full_update: bool
    ) -> Result<Poll, Error> {
        fn date_difference_from_now(date: &ServerTime) -> i64 {
            let current_timestamp = time::get_server_time_now().timestamp();
            
            current_timestamp - date.timestamp()
        }
        
        fn last_poll_full_outdated(last_poll_full_update: Option<ServerTime>) -> bool {
            if let Some(last_poll_full_update) = last_poll_full_update {
                date_difference_from_now(&last_poll_full_update) >= 5 * 60
            } else {
                true
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
                    return Err(Error::PollCalledTooSoon);
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
                offers_since = poll_offers_since.timestamp() - 1800;
            }
        }

        let historical_cutoff = time::timestamp_to_server_time(offers_since);
        let mut offers = self.api.get_trade_offers(&filter, &Some(historical_cutoff)).await?;
        let mut offers_since: i64 = 0;
        let mut poll: Poll = Vec::new();
        
        if let Some(cancel_duration) = self.cancel_duration {
            let cancel_time = chrono::Utc::now() - cancel_duration;
            let offers_to_cancel = offers
                .iter_mut()
                .filter(|offer| {
                    offer.trade_offer_state == TradeOfferState::Active &&
                    offer.is_our_offer &&
                    offer.time_created < cancel_time
                });
            let cancel_futures = offers_to_cancel
                .map(|offer| async { self.cancel_offer(offer).await })
                .collect::<Vec<_>>();
            
            // cancels all offers older than cancel_time
            // this will also update the state for the offers that were cancelled
            futures::future::join_all(cancel_futures).await;
        }
        
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
    
    async fn save_poll_data(&self) -> Result<(), FileError> {
        // we clone this so we don't hold it across an await
        let poll_data = self.poll_data.read().unwrap().clone();
        let data = serde_json::to_string(&poll_data)?;
        
        file::save_poll_data(&self.steamid, &data).await
    }
}