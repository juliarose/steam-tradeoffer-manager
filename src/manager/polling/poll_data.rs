use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{
    time::{date_difference_from_now, ServerTime},
    types::TradeOfferId,
    enums::TradeOfferState,
};
use chrono::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollData {
    #[serde(default)]
    pub offers_since: Option<ServerTime>,
    #[serde(default)]
    pub last_poll: Option<ServerTime>,
    #[serde(default)]
    pub last_poll_full_update: Option<ServerTime>,
    #[serde(default)]
    pub state_map: HashMap<TradeOfferId, TradeOfferState>,
    #[serde(default, skip_serializing)]
    pub changed: bool,
}

impl PollData {
    pub fn new() -> Self {
        Self {
            offers_since: None,
            last_poll: None,
            last_poll_full_update: None,
            state_map: HashMap::new(),
            changed: false,
        }
    }
    
    pub fn last_full_poll_is_stale(&self, update_interval: &Duration) -> bool {
        if let Some(last_poll_full_update) = self.last_poll_full_update {
            date_difference_from_now(&last_poll_full_update) >= *update_interval
        } else {
            true
        }
    }
    
    pub fn clear_offers(&mut self, tradeofferids_to_remove: &[TradeOfferId]) {
        for tradeofferid in tradeofferids_to_remove {
            self.state_map.remove(tradeofferid);
            self.changed = true;
        }
    }
    
    pub fn set_offers_since(&mut self, date: ServerTime) {
        if self.offers_since != Some(date) {
            self.offers_since = Some(date);
            self.changed = true;
        }
    }
    
    pub fn set_last_poll(&mut self, date: ServerTime) {
        self.last_poll = Some(date);
    }
    
    pub fn set_last_poll_full_update(&mut self, date: ServerTime) {
        if self.last_poll_full_update != Some(date) {
            self.last_poll_full_update = Some(date);
            self.changed = true;
        }
    }
}