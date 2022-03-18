use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{
    time::ServerTime,
    types::TradeOfferId,
    TradeOfferState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollData {
    pub offers_since: Option<ServerTime>,
    pub last_poll: Option<ServerTime>,
    pub last_poll_full_update: Option<ServerTime>,
    pub state_map: HashMap<TradeOfferId, TradeOfferState>,
}