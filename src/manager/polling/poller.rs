use super::{file, PollData, PollType};
use crate::api::request::GetTradeOffersOptions;
use crate::api::SteamTradeOfferAPI;
use crate::enums::TradeOfferState;
use crate::error::Error;
use crate::response::TradeOffer;
use crate::time;
use crate::types::TradeOfferId;
use std::collections::{HashMap, HashSet};
use chrono::Duration;
use steamid_ng::SteamID;

/// A poll containing new offers. For each item in the vector, the first element is the
/// [`TradeOffer`]. The second part is the previous [`TradeOfferState`] if this is not a newly
/// encountered offer.
pub type Poll = Vec<(TradeOffer, Option<TradeOfferState>)>;
/// The result of a poll.
pub type Result = std::result::Result<Poll, Error>;

const OFFERS_SINCE_BUFFER_SECONDS: i64 = 60 * 30;
const OFFERS_SINCE_ALL_TIMESTAMP: i64 = 1;

pub struct Poller {
    pub steamid: SteamID,
    pub api: SteamTradeOfferAPI,
    pub cancel_duration: Option<Duration>,
    pub poll_full_update_duration: Duration,
    pub poll_data: PollData,
}

impl Poller {
    /// Performs a poll for changes to offers. `poll_type` determines the type of poll to perform.
    pub async fn do_poll(
        &mut self,
        poll_type: PollType,
    ) -> Result {
        let now = time::get_server_time_now();
        let mut offers_since = self.poll_data.offers_since
            // Steam can be dumb and backdate a modified offer. We need to handle this by adding a buffer.
            .map(|date| date.timestamp() - OFFERS_SINCE_BUFFER_SECONDS)
            .unwrap_or(OFFERS_SINCE_ALL_TIMESTAMP);
        let mut active_only = true;
        let mut is_full_update = {
            poll_type.is_full_update() || 
            // The date of the last full poll is outdated.
            self.poll_data.last_full_poll_is_stale(&self.poll_full_update_duration)
        };
        
        if poll_type == PollType::NewOffers {
            // a very high date
            offers_since = u32::MAX as i64;
            is_full_update = false;
        } else if let PollType::OffersSince(date) = poll_type {
            offers_since = date.timestamp();
            active_only = false;
            is_full_update = false;
        } else if is_full_update {
            offers_since = OFFERS_SINCE_ALL_TIMESTAMP;
            active_only = false;
        }
        
        let (
            mut offers,
            descriptions,
        ) = self.api.get_raw_trade_offers(&GetTradeOffersOptions {
            active_only,
            historical_only: false,
            get_sent_offers: true,
            get_received_offers: true,
            get_descriptions: poll_type.is_active_only(),
            historical_cutoff: Some(time::timestamp_to_server_time(offers_since)),
        }).await?;
        
        if !poll_type.is_active_only() {
            self.poll_data.set_last_poll(now);
        }
        
        if is_full_update {
            self.poll_data.set_last_poll_full_update(now);
        }
        
        // Vec of offers that were cancelled.
        let cancelled_offers = if let Some(cancel_duration) = self.cancel_duration {
            let cancel_time = chrono::Utc::now() - cancel_duration;
            // Cancels all offers older than cancel_time.
            let cancel_futures = offers
                .iter_mut()
                .filter(|offer| {
                    let is_active_state = {
                        offer.trade_offer_state == TradeOfferState::Active ||
                        offer.trade_offer_state == TradeOfferState::CreatedNeedsConfirmation
                    };
                    
                    is_active_state &&
                    offer.is_our_offer &&
                    offer.time_updated < cancel_time &&
                    // offers with a tradeid are in progress and cannot be cancelled
                    offer.tradeid.is_none()
                })
                .map(|offer| self.api.cancel_offer(offer.tradeofferid))
                .collect::<Vec<_>>();
            
            futures::future::join_all(cancel_futures).await
                .into_iter()
                .filter_map(|offer| offer.ok())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        // For reducing file writes, keep track of whether the state of poll data has changed.
        let mut prev_states_map: HashMap<TradeOfferId, TradeOfferState> = HashMap::new();
        let mut poll: Vec<_> = Vec::new();
        let mut offers_since = self.poll_data.offers_since
            .unwrap_or_else(|| time::timestamp_to_server_time(offers_since));
        // Tradeofferids to retain when evicting items from the state map.
        let mut retained_tradeofferids = HashSet::with_capacity(offers.len());
        
        for mut offer in offers {
            // This offer was successfully cancelled above...
            // We need to update its state here.
            if cancelled_offers.contains(&offer.tradeofferid) {
                offer.trade_offer_state = TradeOfferState::Canceled;
            }
            
            // No need to insert into the state map if this isn't a full update.
            if !is_full_update {
                retained_tradeofferids.insert(offer.tradeofferid);
            }
            
            // Just don't do anything with this offer.
            if offer.is_glitched() {
                continue;
            }
            
            // Update the offers_since to the most recent trade offer.
            if offer.time_updated > offers_since {
                offers_since = offer.time_updated;
            }
            
            match self.poll_data.state_map.get(&offer.tradeofferid) {
                // State has changed.
                Some(
                    poll_trade_offer_state
                ) if *poll_trade_offer_state != offer.trade_offer_state => {
                    prev_states_map.insert(offer.tradeofferid, *poll_trade_offer_state);
                    poll.push(offer);
                },
                // Nothing has changed...
                Some(_) => {},
                // This is a new offer
                None => poll.push(offer),
            }
        }
        
        if !poll_type.is_active_only() {
            self.poll_data.set_offers_since(offers_since);
        }
        
        // Trim the state map so it does not grow indefinitely.
        if is_full_update && !retained_tradeofferids.is_empty() {
            self.poll_data.retain_offers(&retained_tradeofferids);
        }
        
        // Maps raw offers to offers with classinfo descriptions.
        let offers = if let Some(descriptions) = descriptions {
            self.api.map_raw_trade_offers_with_descriptions(poll, descriptions)
        } else {
            self.api.map_raw_trade_offers(poll).await?
        };
        let poll = if offers.is_empty() {
            // map_raw_trade_offers may have excluded some offers - the state of the poll data
            // is not updated until all descriptions are loaded for the offer
            Vec::new()
        } else {
            self.poll_data.changed = true;
            offers
                .into_iter()
                // Combines changed state maps.
                .map(|offer| {
                    let prev_state = prev_states_map.remove(&offer.tradeofferid);
                    
                    // insert new state into map
                    self.poll_data.state_map.insert(offer.tradeofferid, offer.trade_offer_state);
                    
                    (offer, prev_state)
                })
                .collect::<Vec<_>>()
        };
        
        // Only save if changes were detected.
        if self.poll_data.changed {
            self.poll_data.changed = false;
            // This could be saved in a background task, but for simplicity, we await here.
            // Saving the file takes a negligible amount of time (usually under a ms on an SSD).
            let _ = file::save_poll_data(
                self.steamid,
                &serde_json::to_string(&self.poll_data)?,
                &self.api.data_directory,
            ).await;
        }
        
        Ok(poll)
    }
}
