use serde::{Serialize, Deserialize};
use crate::{
    serializers::string,
    response::Asset,
    types::{AppId, ContextId, AssetId, Amount},
};

/// An item to send in a trade offer.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Item {
    pub appid: AppId,
    #[serde(with = "string")]
    pub contextid: ContextId,
    #[serde(with = "string")]
    pub assetid: AssetId,
    pub amount: Amount,
}

impl From<Asset> for Item {
    fn from(asset: Asset) -> Item {
        Item {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        }
    }
}

impl From<&Asset> for Item {
    fn from(asset: &Asset) -> Item {
        Item {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        }
    }
}