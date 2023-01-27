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
    /// The ID for this offer.
    pub tradeofferid: TradeOfferId,
    /// The trade ID for this offer. This should be present when the state of the offer is
    /// "Accepted".
    pub tradeid: Option<TradeId>,
    /// The [`SteamID`] of our partner.
    pub partner: SteamID,
    /// The message included in the offer. If the message is empty or not present this will
    /// be `None`.
    pub message: Option<String>,
    /// The items we're receiving in this offer.
    pub items_to_receive: Vec<Asset>,
    /// The items we're giving in this offer.
    pub items_to_give: Vec<Asset>,
    /// Whether this offer was created by us or not.
    pub is_our_offer: bool,
    /// Whether this offer originated from a real time trade.
    pub from_real_time_trade: bool,
    /// The time before the offer expires if it has not been acted on.
    pub expiration_time: ServerTime,
    /// The time this offer was created.
    pub time_created: ServerTime,
    /// The time this offer last had an action e.g. accepting or declining the offer.
    pub time_updated: ServerTime,
    /// The state of this offer.
    pub trade_offer_state: TradeOfferState,
    /// The end date if this trade is in escrow. `None` when this offer is not in escrow.
    pub escrow_end_date: Option<ServerTime>,
    /// The confirmation method for this offer.
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
            escrow_end_date: None,
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
    /// Creates a new [~TradeOffer`].
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