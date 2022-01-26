use steamid_ng::SteamID;
use serde::{Serialize, Deserialize};
use crate::serializers::string;
use crate::types::{
    AppId,
    ContextId,
    AssetId,
    Amount
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateTradeOfferItem {
    pub appid: AppId,
    #[serde(with = "string")]
    pub contextid: ContextId,
    #[serde(with = "string")]
    pub assetid: AssetId,
    pub amount: Amount,
}

pub struct CreateTradeOffer {
    pub id: Option<u64>,
    pub partner: SteamID,
    pub items_to_give: Vec<CreateTradeOfferItem>,
    pub items_to_receive: Vec<CreateTradeOfferItem>,
    pub message: Option<String>,
    pub token: Option<String>,
}