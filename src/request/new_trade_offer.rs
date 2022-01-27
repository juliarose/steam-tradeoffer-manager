use steamid_ng::SteamID;
use serde::{Serialize, Deserialize};
use crate::{
    SteamTradeOfferAPI,
    APIError,
    serializers::string,
    response::{
        SentOffer,
        Asset
    },
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

impl From<Asset> for NewTradeOfferItem {
    
    fn from(asset: Asset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        }
    }
}

pub struct NewTradeOffer<'a> {
    pub api: &'a SteamTradeOfferAPI,
    pub id: Option<TradeOfferId>,
    pub partner: SteamID,
    pub items_to_give: Vec<NewTradeOfferItem>,
    pub items_to_receive: Vec<NewTradeOfferItem>,
    pub message: Option<String>,
    pub token: Option<String>,
}

impl<'a> NewTradeOffer<'a> {

    pub async fn send(&'a self) -> Result<SentOffer, APIError> {
        self.api.send_offer(self).await
    }
}