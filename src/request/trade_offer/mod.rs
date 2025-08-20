mod builder;
mod item;

pub use builder::NewTradeOfferBuilder;
pub use item::NewTradeOfferItem;

use crate::response::{Asset, TradeOffer};
use steamid_ng::SteamID;

/// Represents a trade offer not yet sent. A template including items from an existing offer can
/// be created by calling `NewTradeOffer::from(offer)` on the owned or borrowed [`TradeOffer`].
/// 
/// For constructing offers with a blank starting point, use [`NewTradeOffer::builder`].
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct NewTradeOffer {
    /// The partner's [`SteamID`] for this offer.
    pub partner: SteamID,
    /// The items to give in this offer.
    pub items_to_give: Vec<NewTradeOfferItem>,
    /// The items to received in this offer.
    pub items_to_receive: Vec<NewTradeOfferItem>,
    /// The message to send in this offer.
    pub message: Option<String>,
    /// The token for sending an offer if you are not friends with the partner.
    pub token: Option<String>,
}

impl NewTradeOffer {
    /// The builder for creating a [`NewTradeOffer`].
    pub fn builder(partner: SteamID) -> NewTradeOfferBuilder {
        NewTradeOfferBuilder::new(partner)
    }
    
    /// Checks if any items are included in the offer.
    pub fn is_empty(&self) -> bool {
        self.items_to_give.is_empty() &&
        self.items_to_receive.is_empty()
    }
}

impl From<NewTradeOfferBuilder> for NewTradeOffer {
    fn from(builder: NewTradeOfferBuilder) -> Self {
        Self {
            partner: builder.partner,
            items_to_give: builder.items_to_give,
            items_to_receive: builder.items_to_receive,
            message: builder.message,
            token: builder.token,
        }
    }
}

impl From<&TradeOffer> for NewTradeOffer {
    fn from(offer: &TradeOffer) -> Self {
        Self {
            partner: offer.partner,
            items_to_give: from_trade_offer_items(&offer.items_to_give),
            items_to_receive: from_trade_offer_items(&offer.items_to_receive),
            message: None,
            token: None,
        }
    }
}

impl From<TradeOffer> for NewTradeOffer {
    fn from(offer: TradeOffer) -> Self {
        Self {
            partner: offer.partner,
            items_to_give: from_trade_offer_items(&offer.items_to_give),
            items_to_receive: from_trade_offer_items(&offer.items_to_receive),
            message: None,
            token: None,
        }
    }
}

fn from_trade_offer_items(items: &[Asset]) -> Vec<NewTradeOfferItem> {
    items
        .iter()
        .map(|item| item.into())
        .collect()
}
