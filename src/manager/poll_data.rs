use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{
    time::ServerTime,
    types::TradeOfferId,
    enums::TradeOfferState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollData {
    pub offers_since: Option<ServerTime>,
    pub last_poll: Option<ServerTime>,
    pub last_poll_full_update: Option<ServerTime>,
    /// The oldest active offer. Polling will go back to this time. Used for full update and 
    /// includes offers in escrow.
    pub oldest_active_offer: Option<ServerTime>,
    pub state_map: HashMap<TradeOfferId, TradeOfferState>,
}

impl PollData {
    pub fn new() -> Self {
        Self {
            offers_since: None,
            last_poll: None,
            last_poll_full_update: None,
            oldest_active_offer: None,
            state_map: HashMap::new(),
        }
    }
}