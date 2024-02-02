use crate::api::response::RawAsset;
use crate::response::Asset;
use crate::types::{AppId, ContextId, AssetId, Amount};
use crate::serialize;
use serde::{Serialize, Deserialize};

/// An item to send in a trade offer.
#[derive(Deserialize, Serialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct NewTradeOfferItem {
    /// The app ID e.g. `440` for Team Fortress 2 or `730` for Counter-Strike Global Offensive.
    pub appid: AppId,
    #[serde(with = "serialize::string")]
    /// The context ID.
    pub contextid: ContextId,
    #[serde(with = "serialize::string")]
    /// The unique asset ID. This value is unique to the item's `appid` and `contextid`.
    pub assetid: AssetId,
    /// The amount. If this item is not stackable the amount will be `1`.
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

impl From<&Asset> for NewTradeOfferItem {
    fn from(asset: &Asset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        }
    }
}

impl From<RawAsset> for NewTradeOfferItem {
    fn from(asset: RawAsset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        }
    }
}

impl From<&RawAsset> for NewTradeOfferItem {
    fn from(asset: &RawAsset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        }
    }
}