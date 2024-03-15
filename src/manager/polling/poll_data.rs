use crate::time::{date_difference_from_now, ServerTime};
use crate::types::TradeOfferId;
use crate::enums::TradeOfferState;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use chrono::Duration;

/// Used for storing account poll data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PollData {
    #[serde(default)]
    /// Where to fetch offers since the last poll.
    pub offers_since: Option<ServerTime>,
    #[serde(default)]
    /// The date of the last poll.
    pub last_poll: Option<ServerTime>,
    #[serde(default)]
    /// The last full update.
    pub last_poll_full_update: Option<ServerTime>,
    #[serde(default)]
    /// The state map for trade offers.
    pub state_map: HashMap<TradeOfferId, TradeOfferState>,
    #[serde(default, skip_serializing)]
    /// Whether the data has changed. Used for reducing file writes.
    pub changed: bool,
}

impl PollData {
    /// Checks if the last full poll is stale based on the `update_interval`.
    pub fn last_full_poll_is_stale(&self, update_interval: &Duration) -> bool {
        if let Some(last_poll_full_update) = self.last_poll_full_update {
            if let Some(difference) = date_difference_from_now(&last_poll_full_update) {
                difference >= *update_interval
            } else {
                log::warn!("Failed to get date difference using time {last_poll_full_update}");
                false
            }
        } else {
            true
        }
    }
    
    /// Retains offers in the state map.
    pub fn retain_offers(&mut self, tradeofferids_to_retain: &HashSet<TradeOfferId>) {
        let length = self.state_map.len();
        
        self.state_map.retain(|tradeofferid, _| tradeofferids_to_retain.contains(tradeofferid));
        // If the length of the map has changed, then the state has changed.
        self.changed = self.changed || self.state_map.len() != length;
    }
    
    /// Updates the `offers_since` value.
    pub fn set_offers_since(&mut self, date: ServerTime) {
        if self.offers_since != Some(date) {
            self.offers_since = Some(date);
            self.changed = true;
        }
    }
    
    /// Updates the `last_poll` value.
    pub fn set_last_poll(&mut self, date: ServerTime) {
        self.last_poll = Some(date);
    }
    
    /// Updates the `last_poll_full_update` value.
    pub fn set_last_poll_full_update(&mut self, date: ServerTime) {
        if self.last_poll_full_update != Some(date) {
            self.last_poll_full_update = Some(date);
            self.changed = true;
        }
    }
}