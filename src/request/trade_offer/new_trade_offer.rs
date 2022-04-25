use steamid_ng::SteamID;
use crate::{response, types::TradeOfferId};
use super::{Item, NewTradeOfferBuilder};

#[derive(Debug, Clone, PartialEq)]
pub struct NewTradeOffer {
    pub id: Option<TradeOfferId>,
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
    
    pub fn is_empty(&self) -> bool {
        self.items_to_give.is_empty() &&
        self.items_to_receive.is_empty()
    }
}

impl From<&response::trade_offer::TradeOffer> for NewTradeOffer {
    
    fn from(offer: &response::trade_offer::TradeOffer) -> Self {
        Self {
            id: Some(offer.tradeofferid),
            partner: offer.partner,
            items_to_give: from_trade_offer_items(&offer.items_to_give),
            items_to_receive: from_trade_offer_items(&offer.items_to_receive),
            message: None,
            token: None,
        }
    }
}

impl From<response::trade_offer::TradeOffer> for NewTradeOffer {
    
    fn from(offer: response::trade_offer::TradeOffer) -> Self {
        Self {
            id: Some(offer.tradeofferid),
            partner: offer.partner,
            items_to_give: from_trade_offer_items(&offer.items_to_give),
            items_to_receive: from_trade_offer_items(&offer.items_to_receive),
            message: None,
            token: None,
        }
    }
}

fn from_trade_offer_items(items: &[response::asset::Asset]) -> Vec<Item> {
    items.iter()
        .map(|item| item.into())
        .collect::<Vec<_>>()
}