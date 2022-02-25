use std::fmt;
use crate::{
    time::ServerTime,
    TradeOfferState,
    ConfirmationMethod,
    types::TradeOfferId,
};
use super::asset::Asset;
use steamid_ng::SteamID;

#[derive(Debug)]
pub struct TradeOffer {
    pub tradeofferid: TradeOfferId,
    pub partner: SteamID,
    pub message: Option<String>,
    pub items_to_receive: Vec<Asset>,
    pub items_to_give: Vec<Asset>,
    pub is_our_offer: bool,
    pub from_real_time_trade: bool,
    pub expiration_time: ServerTime,
    pub time_created: ServerTime,
    pub time_updated: ServerTime,
    pub trade_offer_state: TradeOfferState,
    pub escrow_end_date: ServerTime,
    pub confirmation_method: ConfirmationMethod,
}

impl fmt::Display for TradeOffer {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", u64::from(self.partner), self.tradeofferid)
    }
}

impl TradeOffer {
    
    pub fn is_glitched(&self) -> bool {
        self.items_to_receive.len() == 0 && self.items_to_receive.len() == 0
    }
}