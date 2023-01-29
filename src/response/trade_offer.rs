use std::fmt;
use crate::{
    SteamID,
    time::ServerTime,
    enums::{TradeOfferState, ConfirmationMethod},
    types::{TradeId, TradeOfferId},
    serialize::{string, option_string},
};
use serde::{Deserialize, Serialize};
use chrono::serde::{ts_seconds, ts_seconds_option};
use super::asset::Asset;

/// A trade offer.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeOffer {
    #[serde(with = "string")]
    /// The ID for this offer.
    pub tradeofferid: TradeOfferId,
    #[serde(with = "option_string")]
    /// The trade ID for this offer. This should be present when the `trade_offer_state` of this 
    /// offer is [`TradeOfferState::Accepted`].
    pub tradeid: Option<TradeId>,
    /// The [`SteamID`] of our partner.
    pub partner: SteamID,
    /// The message included in the offer. If the message is empty or not present this will be 
    /// `None`.
    pub message: Option<String>,
    /// The items we're receiving in this offer.
    pub items_to_receive: Vec<Asset>,
    /// The items we're giving in this offer.
    pub items_to_give: Vec<Asset>,
    /// Whether this offer was created by us or not.
    pub is_our_offer: bool,
    /// Whether this offer originated from a real time trade.
    pub from_real_time_trade: bool,
    #[serde(with = "ts_seconds")]
    /// The time this offer was created.
    pub time_created: ServerTime,
    #[serde(with = "ts_seconds")]
    /// The time before the offer expires if it has not been acted on.
    pub expiration_time: ServerTime,
    #[serde(with = "ts_seconds")]
    /// The time this offer last was last acted on e.g. accepting or declining the offer.
    pub time_updated: ServerTime,
    /// The state of this offer.
    pub trade_offer_state: TradeOfferState,
    #[serde(with = "ts_seconds_option")]
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
}