use std::fmt;
use crate::{
    SteamID,
    time::ServerTime,
    enums::{
        TradeOfferState,
        ConfirmationMethod,
    },
    types::{TradeId, TradeOfferId},
};
use super::asset::Asset;

/// A trade offer.
#[derive(Debug)]
pub struct TradeOffer {
    pub tradeofferid: TradeOfferId,
    pub tradeid: Option<TradeId>,
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

impl Default for TradeOffer {
    
    fn default() -> Self {
        TradeOffer {
            tradeofferid: 0,
            tradeid: None,
            partner: SteamID::from(0),
            message: None,
            items_to_receive: Vec::new(),
            items_to_give: Vec::new(),
            is_our_offer: false,
            from_real_time_trade: false,
            expiration_time: chrono::Utc::now(),
            time_created: chrono::Utc::now(),
            time_updated: chrono::Utc::now(),
            trade_offer_state: TradeOfferState::Active,
            escrow_end_date: chrono::Utc::now(),
            confirmation_method: ConfirmationMethod::None,
        }
    }
}

impl fmt::Display for TradeOffer {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", u64::from(self.partner), self.tradeofferid)
    }
}

impl TradeOffer {
    
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Checks whether the trade offer is glitched or not by checking if no items are present.
    pub fn is_glitched(&self) -> bool {
        self.items_to_receive.is_empty() && self.items_to_give.is_empty()
    }
    
    /// Whether the state of this offer can be modified. This is either active offers or offers 
    /// that are in escrow.
    pub fn state_is_changeable(&self) -> bool {
        self.trade_offer_state == TradeOfferState::Active ||
        self.trade_offer_state == TradeOfferState::InEscrow ||
        self.trade_offer_state == TradeOfferState::CreatedNeedsConfirmation
    }
}