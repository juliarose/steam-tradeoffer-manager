mod item;
mod builder;

pub use item::Item;
pub use builder::NewTradeOfferBuilder;
use steamid_ng::SteamID;
use crate::response;

/// Represents a trade offer not yet sent. A template including items from an existing offer can
/// created by calling `from` on the offer.
#[derive(Debug, Clone, PartialEq)]
pub struct NewTradeOffer {
    pub partner: SteamID,
    pub items_to_give: Vec<Item>,
    pub items_to_receive: Vec<Item>,
    pub message: Option<String>,
    pub token: Option<String>,
}

impl NewTradeOffer {
    pub fn builder(steamid: SteamID) -> NewTradeOfferBuilder {
        NewTradeOfferBuilder::new(steamid)
    }
    
    /// Checks if any items are included in the offer.
    pub fn is_empty(&self) -> bool {
        self.items_to_give.is_empty() &&
        self.items_to_receive.is_empty()
    }
}

impl From<&response::TradeOffer> for NewTradeOffer {
    fn from(offer: &response::TradeOffer) -> Self {
        Self {
            partner: offer.partner,
            items_to_give: from_trade_offer_items(&offer.items_to_give),
            items_to_receive: from_trade_offer_items(&offer.items_to_receive),
            message: None,
            token: None,
        }
    }
}

impl From<response::TradeOffer> for NewTradeOffer {
    fn from(offer: response::TradeOffer) -> Self {
        Self {
            partner: offer.partner,
            items_to_give: from_trade_offer_items(&offer.items_to_give),
            items_to_receive: from_trade_offer_items(&offer.items_to_receive),
            message: None,
            token: None,
        }
    }
}

fn from_trade_offer_items(items: &[response::Asset]) -> Vec<Item> {
    items.iter()
        .map(|item| item.into())
        .collect::<Vec<_>>()
}