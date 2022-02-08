use crate::{response, TradeOfferState};

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