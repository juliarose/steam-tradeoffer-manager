use serde::{Serialize, Deserialize};
use std::sync::Arc;
use super::{TradeAsset, ClassInfo};
use crate::{
    serialize::string,
    types::{AppId, ContextId, AssetId, Amount},
    internal_types::ClassInfoClass,
};

/// An asset which includes its related [`ClassInfo`] mapping.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Asset {
    /// The app ID e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    pub appid: AppId,
    /// The context ID.
    #[serde(with = "string")]
    pub contextid: ContextId,
    /// The unique asset ID. This value is unique to the item's `appid` and `contextid`.
    #[serde(with = "string")]
    pub assetid: AssetId,
    /// The amount. If this item is not stackable the amount will be `1`.
    #[serde(with = "string")]
    pub amount: Amount,
    /// The [`ClassInfo`] containing names, descriptions, and other details about the item.
    pub classinfo: Arc<ClassInfo>,
}

impl Asset {
    /// The key used for [`ClassInfo`] data.
    pub fn class(&self) -> ClassInfoClass {
        (self.appid, self.classinfo.classid, self.classinfo.instanceid)
    }
}

/// Converts a [`TradeAsset`] into an [`Asset`]. The `contextid` and `assetid` are taken from 
/// `contextid` and `assetid` respectively, **not** `new_contextid` and `new_assetid`.
/// 
/// If you need an [`Asset`] of the newly acquired item, call `try_into_new_asset` on the
/// [`TradeAsset`].
impl From<TradeAsset> for Asset {
    fn from(trade_asset: TradeAsset) -> Self {
        Asset {
            appid: trade_asset.appid,
            contextid: trade_asset.contextid,
            assetid: trade_asset.assetid,
            amount: trade_asset.amount,
            classinfo: trade_asset.classinfo,
        }
    }
}

/// Converts a borrowed [`TradeAsset`] into an [`Asset`]. The `contextid` and `assetid` are taken 
/// from `contextid` and `assetid` respectively, **not** `new_contextid` and `new_assetid`.
/// 
/// If you need an [`Asset`] of the newly acquired item, call `try_into_new_asset` on the
/// [`TradeAsset`].
impl From<&TradeAsset> for Asset {
    fn from(trade_asset: &TradeAsset) -> Self {
        Asset {
            appid: trade_asset.appid,
            contextid: trade_asset.contextid,
            assetid: trade_asset.assetid,
            amount: trade_asset.amount,
            classinfo: Arc::clone(&trade_asset.classinfo),
        }
    }
}