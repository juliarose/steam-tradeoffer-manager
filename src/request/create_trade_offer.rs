use crate::Item;
use steamid_ng::SteamID;

pub struct CreateTradeOffer {
    pub id: Option<u64>,
    pub partner: SteamID,
    pub items_to_give: Vec<Item>,
    pub items_to_receive: Vec<Item>,
    pub message: Option<String>,
    pub token: Option<String>,
}