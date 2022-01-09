use crate::{
    time::ServerTime,
    Item,
    TradeOfferState
};
use steamid_ng::SteamID;

pub struct TradeOffer {
    pub partner: SteamID,
    pub id: u64,
    pub state: TradeOfferState,
    pub items_to_give: Vec<Item>,
    pub items_to_receive: Vec<Item>,
    pub message: Option<String>,
    pub created: ServerTime,
    pub updated: ServerTime,
    pub expires: ServerTime,
    pub trade_id: Option<String>,
    pub is_our_offer: bool,
    pub from_real_time_trade: bool,
    pub confirmation_method: u32,
}

impl TradeOffer {

}