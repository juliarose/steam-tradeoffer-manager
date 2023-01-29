mod item;
mod builder;

pub use item::NewTradeOfferItem;
pub use builder::NewTradeOfferBuilder;
use steamid_ng::SteamID;
use crate::response::{TradeOffer, Asset};

/// Represents a trade offer not yet sent. A template including items from an existing offer can
/// created by calling `from` on the owned or borrowed [`TradeOffer`]. For constructing blank 
/// offers [`NewTradeOffer::builder`] to create a new [`NewTradeOfferBuilder`] is useful.
#[derive(Debug, Clone, PartialEq)]
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