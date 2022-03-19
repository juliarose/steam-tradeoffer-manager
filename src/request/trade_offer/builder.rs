use super::{Item, NewTradeOffer};
use crate::{SteamID, types::TradeOfferId};

pub struct NewTradeOfferBuilder {
    pub id: Option<TradeOfferId>,
    pub partner: SteamID,
    pub items_to_give: Vec<Item>,
    pub items_to_receive: Vec<Item>,
    pub message: Option<String>,
    pub token: Option<String>,
}

impl NewTradeOfferBuilder {
    
    pub fn new(partner: SteamID) -> Self {
        Self {
            id: None,
            partner,
            items_to_give: Vec::new(),
            items_to_receive: Vec::new(),
            message: None,
            token: None,
        }
    }

    pub fn counter(mut self, id: TradeOfferId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn items_to_give(mut self, items: Vec<Item>) -> Self {
        self.items_to_give = items;
        self
    }

    pub fn items_to_receive(mut self, items: Vec<Item>) -> Self {
        self.items_to_receive = items;
        self
    }

    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
    
    pub fn build(self) -> NewTradeOffer {
        NewTradeOffer {
            id: self.id,
            partner: self.partner,
            items_to_give: self.items_to_give,
            items_to_receive: self.items_to_receive,
            message: self.message,
            token: self.token,
        }
    }
}