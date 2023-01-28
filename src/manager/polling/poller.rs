
use super::{file, PollData};
use crate::{
    time,
    enums::TradeOfferState,
    types::TradeOfferId,
    response::TradeOffer,
    api::SteamTradeOfferAPI,
    error::Error,
};
use std::{path::PathBuf, collections::HashMap};
use chrono::Duration;
use steamid_ng::SteamID;

pub type Poll = Vec<(TradeOffer, Option<TradeOfferState>)>;
pub type PollResult = Result<Poll, Error>;

/// The type of poll to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollType {
    /// Let the manager decide. Unless you need to fetch offers in special cases this is what 
    /// should be used.
    Auto,
    /// Fastest method for obtaining new offers when new offers. This will fetch only active 
    /// offers and includes descriptions in the response rather than relying on 
    /// ISteamEconomy/GetAssetClassInfo. For this reason, items in the response will also not 
    /// contain app_data. This will not update the timestamps in the poll data. For this reason, 
    /// this should not be used as your only method of polling if you care about checking the 
    /// state of changed offers.
    NewOffers,
    /// Do a full update.
    FullUpdate,
}

impl PollType {
    /// The poll is a full update.
    fn is_full_update(&self) -> bool {
        matches!(self, Self::FullUpdate)
    }
    
    /// The poll is only active offers.
    fn is_active_only(&self) -> bool {
        matches!(self, Self::NewOffers)
    }
}

pub struct Poller {
    pub steamid: SteamID,
    pub api: SteamTradeOfferAPI,
    pub data_directory: PathBuf,
    pub cancel_duration: Option<Duration>,
    pub full_poll_update_duration: Duration,
    pub poll_data: PollData,
}

impl Poller {
    /// Performs a poll for changes to offers. Provides a parameter to determine what type of poll to perform.
    pub async fn do_poll(
        &mut self,
        poll_type: PollType,
    ) -> PollResult {
        let now = time::get_server_time_now();
        let mut offers_since = self.poll_data.offers_since
            // It looks like sometimes Steam can be dumb and backdate a modified offer.
            // We need to handle this. Let's add a 30-minute buffer.
            .map(|date| date.timestamp() - (60 * 30))
            .unwrap_or(1);
        let mut active_only = true;
        let mut full_update = {
            poll_type.is_full_update() || 
            // The date of the last full poll is outdated.
            self.poll_data.last_full_poll_is_stale(&self.full_poll_update_duration)
        };
        
        if poll_type == PollType::NewOffers {
            // a very high date
            offers_since = u32::MAX as i64;
            full_update = false;
        } else if full_update {
            offers_since = 1;
            active_only = false;
        }
        
        let (mut offers, descriptions) = self.api.get_raw_trade_offers(
            active_only,
            false,
            true,
            true,
            poll_type.is_active_only(),
            Some(time::timestamp_to_server_time(offers_since)),
        ).await?;
        
        if !poll_type.is_active_only() {
            self.poll_data.set_last_poll(now);
        }
        
        if full_update {
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
                    offer.time_created < cancel_time
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
        
        for mut offer in offers {
            // This offer was successfully cancelled above...
            // We need to update its state here.
            if cancelled_offers.contains(&offer.tradeofferid) {
                offer.trade_offer_state = TradeOfferState::Canceled;
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
        
        // Clear poll data offers otherwise this could expand infinitely.
        // Using a higher number than is removed so this process needs to run less frequently.
        // This could be better but it works.
        if self.poll_data.state_map.len() > 2500 {
            let mut tradeofferids = self.poll_data.state_map
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            
            // High to low.
            tradeofferids.sort_by(|a, b| b.cmp(a));
            
            let (
                _tradeofferids,
                tradeofferids_to_remove,
            ) = tradeofferids.split_at(2000);
            
            self.poll_data.clear_offers(tradeofferids_to_remove);
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
            // It's really not a problem to await on this.
            // Saving the file takes under a millisecond.
            let _ = file::save_poll_data(
                &self.steamid,
                &serde_json::to_string(&self.poll_data)?,
                &self.data_directory,
            ).await;
        }
        
        Ok(poll)
    }
}