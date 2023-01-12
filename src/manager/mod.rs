mod poll_data;
mod file;
mod builder;

pub use builder::TradeOfferManagerBuilder;
use poll_data::PollData;
use std::{
    cmp,
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};
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

pub type Poll = Vec<(response::TradeOffer, Option<TradeOfferState>)>;
pub const USER_AGENT_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36";

/// Manager which includes functionality for interacting with trade offers, confirmations and 
/// inventories.
#[derive(Debug)]
pub struct TradeOfferManager {
    /// The account's SteamID.
    pub steamid: SteamID,
    pub cancel_duration: Option<Duration>,
    /// The underlying API.
    api: SteamTradeOfferAPI,
    /// The underlying API for mobile confirmations.
    mobile_api: MobileAPI,
    /// Account poll data.
    poll_data: Arc<RwLock<PollData>>,
    /// The directory to store poll data and [`response::ClassInfo`] data.
    data_directory: PathBuf,
}

impl TradeOfferManager {
    /// Creates a new [`TradeOfferManager`].
    pub fn new(
        steamid: SteamID,
        key: String,
    ) -> Self {
        Self::builder(steamid, key).build()
    }
    
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
    
    /// Accepts an offer. This checks if the offer can be acted on and updates the state of the 
    /// offer upon success.
    pub async fn accept_offer(
        &self,
        offer: &mut response::TradeOffer,
    ) -> Result<response::AcceptedOffer, Error> {
        if offer.is_our_offer {
            return Err(Error::Parameter("Cannot accept an offer that is ours"));
        } else if offer.trade_offer_state != TradeOfferState::Active {
            return Err(Error::Parameter("Cannot accept an offer that is not active"));
        }
        
        let accepted_offer = self.api.accept_offer(offer.tradeofferid, &offer.partner).await?;
        offer.trade_offer_state = TradeOfferState::Accepted;
        
        Ok(accepted_offer)
    }
    
    /// Accepts an offer using its tradeofferid..
    pub async fn accept_offer_id(
        &self,
        tradeofferid: TradeOfferId,
        partner: &SteamID,
    ) -> Result<response::AcceptedOffer, Error> {
        let accepted_offer = self.api.accept_offer(tradeofferid, &partner).await?;
        
        Ok(accepted_offer)
    }
    
    /// Cancels an offer. This checks if the offer was not creating by us and updates the state of 
    /// the offer upon success.
    pub async fn cancel_offer(
        &self,
        offer: &mut response::TradeOffer,
    ) -> Result<(), Error> {
        if !offer.is_our_offer {
            return Err(Error::Parameter("Cannot cancel an offer we did not create"));
        }
        
        self.api.cancel_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Canceled;
        
        Ok(())
    }
    
    /// Cancels an offer using its tradeofferid.
    pub async fn cancel_offer_id(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<(), Error> {
        self.api.cancel_offer(tradeofferid).await?;
        
        Ok(())
    }
    
    /// Declines an offer. This checks if the offer was creating by us and updates the state of 
    /// the offer upon success.
    pub async fn decline_offer(
        &self,
        offer: &mut response::TradeOffer,
    ) -> Result<(), Error> {
        if offer.is_our_offer {
            return Err(Error::Parameter("Cannot decline an offer we created"));
        }
        
        self.api.decline_offer(offer.tradeofferid).await?;
        offer.trade_offer_state = TradeOfferState::Declined;
        
        Ok(())
    }
    
    /// Declines an offer using its tradeofferid.
    pub async fn decline_offer_id(
        &self,
        tradeofferid: TradeOfferId,
    ) -> Result<(), Error> {
        self.api.decline_offer(tradeofferid).await?;
        
        Ok(())
    }
    
    /// Sends an offer.
    pub async fn send_offer(
        &self,
        offer: &request::trade_offer::NewTradeOffer,
    ) -> Result<response::SentOffer, Error> {
        self.api.send_offer(offer, None).await
    }
    
    /// Counters an existing offer. This updates the state of the offer upon success.
    pub async fn counter_offer(
        &self,
        offer: &mut response::TradeOffer,
        counter_offer: &request::trade_offer::NewTradeOffer,
    ) -> Result<response::SentOffer, Error> {
        let sent_offer = self.api.send_offer(
            counter_offer,
            Some(offer.tradeofferid),
        ).await?;
        
        offer.trade_offer_state = TradeOfferState::Countered;
        
        Ok(sent_offer)
    }
    
    /// Counters an existing offer using its tradeofferid.
    pub async fn counter_offer_id(
        &self,
        tradeofferid: TradeOfferId,
        counter_offer: &request::trade_offer::NewTradeOffer,
    ) -> Result<response::SentOffer, Error> {
        let sent_offer = self.api.send_offer(
            counter_offer,
            Some(tradeofferid),
        ).await?;
        
        Ok(sent_offer)
    }

    /// Gets a user's inventory using the old endpoint.
    pub async fn get_inventory_old(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::Asset>, Error> {
        self.api.get_inventory_old(steamid, appid, contextid, tradable_only).await
    }
    
    /// Gets a user's inventory.
    pub async fn get_inventory(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::Asset>, Error> {
        self.api.get_inventory(steamid, appid, contextid, tradable_only).await
    }
    
    /// Gets a user's inventory with more detailed clasinfo data using the GetAssetClassInfo API.
    pub async fn get_inventory_with_classinfos(
        &self,
        steamid: &SteamID,
        appid: AppId,
        contextid: ContextId,
        tradable_only: bool,
    ) -> Result<Vec<response::Asset>, Error> {
        self.api.get_inventory_with_classinfos(steamid, appid, contextid, tradable_only).await
    }
    
    /// Gets the user's details for trading.
    pub async fn get_user_details(
        &self,
        tradeofferid: &Option<TradeOfferId>,
        partner: &SteamID,
        token: &Option<String>,
    ) -> Result<response::UserDetails, Error> {
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
    pub async fn get_receipt(&self, offer: &response::TradeOffer) -> Result<Vec<response::Asset>, Error> {
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
    pub async fn update_offer(&self, offer: &mut response::TradeOffer) -> Result<(), Error> {
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
    ) -> Result<Vec<response::TradeOffer>, Error> {
        self.api.get_trade_offers(&OfferFilter::ActiveOnly, &None).await
    }
    
    /// Gets trade offers.
    pub async fn get_trade_offers(
        &self,
        filter: &OfferFilter,
        historical_cutoff: &Option<ServerTime>,
    ) -> Result<Vec<response::TradeOffer>, Error> {
        self.api.get_trade_offers(filter, historical_cutoff).await
    }
    
    /// Forces a pull. This will do a poll without checking whether the last poll occurred 
    /// too recently (returning a [`Error::PollCalledTooSoon`] error). If full_update is false 
    /// this will not do a full update even if the last full update is outdated.
    pub async fn force_do_poll(
        &self,
        full_update: bool,
    ) -> Result<Poll, Error> {
        self.do_poll_request(full_update, true).await
    }
    
    /// Performs a poll for changes to offers. If full_update is set, the poll will get offers up 
    /// to your oldest active offers. A full update will be forced if the last full update was 
    /// more than 5 minutes ago.
    pub async fn do_poll(
        &self,
        full_update: bool,
    ) -> Result<Poll, Error> {
        self.do_poll_request(full_update, false).await
    }
    
    /// Performs a poll for changes to offers. If full_update is set, the poll will get offers up 
    /// to your oldest active offers.
    async fn do_poll_request(
        &self,
        mut full_update: bool,
        force_update: bool,
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
        
        // Updates the oldest active offer. Since this is quite complicated it deserves its 
        // own function...
        fn update_polled_oldest_active_offer(
            full_update: bool,
            poll_data: &PollData,
            offer: &crate::api::raw::RawTradeOffer,
            polled_oldest_active_offer: &mut Option<ServerTime>,
        ) {
            // This offer cannot be changed - we don't care about it.
            if !offer.state_is_changeable() {
                return;
            }
            
            let is_updateable_by_full_update = {
                // Update it only if we're doing a full update.
                full_update &&
                {
                    // And the time of the offer is older than the current oldest active offer.
                    Some(offer.time_created) < *polled_oldest_active_offer ||
                    // Unless the current oldest active offer is not set.
                    polled_oldest_active_offer.is_none()
                }
            };
            
            if {
                is_updateable_by_full_update ||
                // If the poll data does not have an active offer set, this is fine too.
                poll_data.oldest_active_offer.is_none()
            } {
                // This is now the oldest active offer.
                *polled_oldest_active_offer = Some(offer.time_created);
            }
        }
        
        let mut offers_since = 0;
        let mut filter = OfferFilter::ActiveOnly;
        
        {
            let mut poll_data = self.poll_data.write().unwrap();
            
            if !force_update {
                if let Some(last_poll) = poll_data.last_poll {
                    let seconds_since_last_poll = date_difference_from_now(&last_poll);
                        
                    if seconds_since_last_poll <= 1 {
                        // We last polled less than a second ago... we shouldn't spam the API
                        return Err(Error::PollCalledTooSoon);
                    }            
                }
            }
            
            poll_data.last_poll = Some(time::get_server_time_now());
        
            if {
                // If we're doing a full update.
                full_update ||
                {
                    // Or the date of the last full poll is outdated.
                    last_poll_full_outdated(poll_data.last_poll_full_update) &&
                    // Unless force_update is set, then we only want active offers.
                    !force_update
                }
            } {
                filter = OfferFilter::All;
                poll_data.last_poll_full_update = Some(time::get_server_time_now());
                full_update = true;
                
                if let Some(oldest_active_offer) = poll_data.oldest_active_offer {
                    // It looks like sometimes Steam can be dumb and backdate a modified offer.
                    // We need to handle this. Let's add a 30-minute buffer.
                    offers_since = oldest_active_offer.timestamp() - 1800;
                } else {
                    offers_since = 1;
                }
            } else if let Some(poll_offers_since) = poll_data.offers_since {
                // It looks like sometimes Steam can be dumb and backdate a modified offer. We 
                // need to handle this. Let's add a 30-minute buffer.
                offers_since = poll_offers_since.timestamp() - 1800;
            }
        }
        
        let historical_cutoff = time::timestamp_to_server_time(offers_since);
        let mut offers = self.api.get_raw_trade_offers(
            &filter,
            &Some(historical_cutoff),
        ).await?;
        let mut offers_since: i64 = 0;
        let mut cancelled_offers = Vec::new();
        
        if let Some(cancel_duration) = self.cancel_duration {
            let cancel_time = chrono::Utc::now() - cancel_duration;
            let offers_to_cancel = offers
                .iter_mut()
                .filter(|offer| {
                    let is_active_state = {
                        offer.trade_offer_state == TradeOfferState::Active ||
                        offer.trade_offer_state == TradeOfferState::CreatedNeedsConfirmation
                    };
                    
                    is_active_state &&
                    offer.is_our_offer &&
                    offer.time_created < cancel_time
                });
            let cancel_futures = offers_to_cancel
                .map(|offer| async {
                    self.api.cancel_offer(offer.tradeofferid).await
                })
                .collect::<Vec<_>>();
            // Cancels all offers older than cancel_time.
            let results = futures::future::join_all(cancel_futures).await;
            
            cancelled_offers.extend(&mut results
                .into_iter()
                .filter_map(|offer| offer.ok())
            );
        }
        
        // For reducing file writes, keep track of whether the state of poll data has changed.
        let mut poll_data_changed = false;
        let mut prev_states_map: HashMap<TradeOfferId, TradeOfferState> = HashMap::new();
        let mut poll: Vec<_> = Vec::new();
        
        {
            let mut poll_data = self.poll_data.write().unwrap();
            let mut polled_oldest_active_offer: Option<ServerTime> = None;
            
            for mut offer in offers {
                // This offer was successfully cancelled above...
                // We need to update its state here.
                if cancelled_offers.contains(&offer.tradeofferid) {
                    offer.trade_offer_state = TradeOfferState::Canceled;
                }
                
                // Detects the oldest active offer.
                update_polled_oldest_active_offer(
                    full_update,
                    &poll_data,
                    &offer,
                    &mut polled_oldest_active_offer,
                );
                
                // Just don't do anything with this offer.
                if offer.is_glitched() {
                    continue;
                }
                
                offers_since = cmp::max(offers_since, offer.time_updated.timestamp());

                match poll_data.state_map.get(&offer.tradeofferid) {
                    Some(
                        poll_trade_offer_state
                    ) if poll_trade_offer_state != &offer.trade_offer_state => {
                        let tradeofferid = offer.tradeofferid;
                        let new_state = offer.trade_offer_state.clone();
                        
                        poll.push(offer);
                        prev_states_map.insert(tradeofferid, *poll_trade_offer_state);
                        poll_data.state_map.insert(tradeofferid, new_state);
                        poll_data_changed = true;
                    },
                    // Nothing has changed...
                    Some(_) => {},
                    None => {
                        // This is a new offe.r
                        poll_data.state_map.insert(offer.tradeofferid, offer.trade_offer_state.clone());
                        poll.push(offer);
                        poll_data_changed = true;
                    },
                }
            }
            
            if polled_oldest_active_offer.is_some() {
                poll_data.oldest_active_offer = polled_oldest_active_offer;
                poll_data_changed = true;
            }
            
            // Clear poll data offers otherwise this could expand infinitely.
            // Using a higher number than is removed so this process needs to run less frequently.
            // This could be better but it works.
            if poll_data.state_map.len() > 2500 {
                let mut tradeofferids = poll_data.state_map
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>();
                
                // High to low.
                tradeofferids.sort_by(|a, b| b.cmp(a));
                
                let (
                    _tradeofferids,
                    tradeofferids_to_remove,
                ) = tradeofferids.split_at(2000);
                
                for tradeofferid in tradeofferids_to_remove {
                    poll_data.state_map.remove(tradeofferid);
                    poll_data_changed = true;
                }
            }
            
            let new_offers_since = Some(time::timestamp_to_server_time(offers_since));
            
            if
                offers_since > 0 &&
                {
                    new_offers_since > poll_data.offers_since ||
                    poll_data.offers_since.is_none() 
                }
            {
                poll_data.offers_since = new_offers_since;
                poll_data_changed = true;
            }
        }
        
        if poll_data_changed {
            // Only save if changes were detected.
            let _ = self.save_poll_data().await;
        }
        
        // Maps raw offers to offers with classinfo descriptions.
        let poll = self.api.map_raw_trade_offers(poll).await?
            .into_iter()
            // Combines changed state maps.
            .map(|offer| {
                let prev_state = prev_states_map.remove(&offer.tradeofferid);
                
                (offer, prev_state)
            })
            .collect::<Vec<_>>();
        
        Ok(poll)
    }
    
    async fn save_poll_data(&self) -> Result<(), FileError> {
        // we clone this so we don't hold it across an await
        let poll_data = self.poll_data.read().unwrap().clone();
        let data = serde_json::to_string(&poll_data)?;
        
        file::save_poll_data(
            &self.steamid,
            &data,
            &self.data_directory,
        ).await
    }
}