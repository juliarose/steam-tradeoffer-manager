use serde::{Serialize, Deserialize};
use crate::{
    serialize::string,
    api::{RawTradeAsset, RawAsset},
    response::{TradeAsset, Asset},
    types::{AppId, ContextId, AssetId, Amount},
};

/// An item to send in a trade offer.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct NewTradeOfferItem {
    /// The app ID e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    pub appid: AppId,
    #[serde(with = "string")]
    /// The context ID.
    pub contextid: ContextId,
    #[serde(with = "string")]
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

/// Converts a [`TradeAsset`] into an [`NewTradeOfferItem`]. The `contextid` and `assetid` are 
/// taken from `new_contextid` and `new_assetid` respectively.
impl From<TradeAsset> for NewTradeOfferItem {
    fn from(trade_asset: TradeAsset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: trade_asset.appid,
            contextid: trade_asset.new_contextid,
            assetid: trade_asset.new_assetid,
            amount: trade_asset.amount,
        }
    }
}

/// Converts a borrowed [`TradeAsset`] into an [`NewTradeOfferItem`]. The `contextid` and 
/// `assetid` are taken from `new_contextid` and `new_assetid` respectively.
impl From<&TradeAsset> for NewTradeOfferItem {
    fn from(asset: &TradeAsset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: asset.appid,
            contextid: asset.new_contextid,
            assetid: asset.new_contextid,
            amount: asset.amount,
        }
    }
}

/// Converts a [`RawTradeAsset`] into an [`NewTradeOfferItem`]. The `contextid` and `assetid` are 
/// taken from `new_contextid` and `new_assetid` respectively.
impl From<RawTradeAsset> for NewTradeOfferItem {
    fn from(raw_trade_asset: RawTradeAsset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: raw_trade_asset.appid,
            contextid: raw_trade_asset.new_contextid,
            assetid: raw_trade_asset.new_assetid,
            amount: raw_trade_asset.amount,
        }
    }
}

/// Converts a borrowed [`RawTradeAsset`] into an [`NewTradeOfferItem`]. The `contextid` and 
/// `assetid` are taken from `new_contextid` and `new_assetid` respectively.
impl From<&RawTradeAsset> for NewTradeOfferItem {
    fn from(raw_trade_asset: &RawTradeAsset) -> NewTradeOfferItem {
        NewTradeOfferItem {
            appid: raw_trade_asset.appid,
            contextid: raw_trade_asset.new_contextid,
            assetid: raw_trade_asset.new_contextid,
            amount: raw_trade_asset.amount,
        }
    }
}