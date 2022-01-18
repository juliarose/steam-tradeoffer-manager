use crate::{
    time::ServerTime,
    TradeOfferState,
    ConfirmationMethod
};
use super::Asset;
use steamid_ng::SteamID;

#[derive(Debug)]
pub struct TradeOffer {
    pub tradeofferid: u64,
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

impl TradeOffer {

}