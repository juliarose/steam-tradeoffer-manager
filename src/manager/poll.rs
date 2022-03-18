use crate::{response, TradeOfferState};

pub type Poll = Vec<(response::trade_offer::TradeOffer, Option<TradeOfferState>)>;

#[derive(Debug)]
pub struct PollChange {
    pub old_state: TradeOfferState,
    pub new_state: TradeOfferState,
    pub offer: response::trade_offer::TradeOffer,
}