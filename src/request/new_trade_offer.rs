use steamid_ng::SteamID;
use serde::{Serialize, Deserialize};
use crate::{
    response,
    serializers::string,
    types::{
        AppId,
        ContextId,
        AssetId,
        Amount,
        TradeOfferId
    }
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewTradeOfferItem {
    pub appid: AppId,
    #[serde(with = "string")]
    pub contextid: ContextId,
    #[serde(with = "string")]
    pub assetid: AssetId,
    pub amount: Amount,
}

impl From<response::asset::Asset> for NewTradeOfferItem {
    
    fn from(asset: response::asset::Asset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        }
    }
}

impl From<&response::asset::Asset> for NewTradeOfferItem {
    
    fn from(asset: &response::asset::Asset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        }
    }
}

pub struct NewTradeOffer {
    pub id: Option<TradeOfferId>,
    pub partner: SteamID,
    pub items_to_give: Vec<NewTradeOfferItem>,
    pub items_to_receive: Vec<NewTradeOfferItem>,
    pub message: Option<String>,
    pub token: Option<String>,
}
