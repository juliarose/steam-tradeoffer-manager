use crate::{response, TradeOfferState};

// #[derive(Debug)]
// pub struct Poll {
//     pub new: Vec<response::trade_offer::TradeOffer>,
//     pub changed: Vec<PollChange>,
// }

pub type Poll = Vec<(response::trade_offer::TradeOffer, Option<TradeOfferState>)>;

#[derive(Debug)]
pub struct PollChange {
    pub old_state: TradeOfferState,
    pub new_state: TradeOfferState,
    pub offer: response::trade_offer::TradeOffer,
}