use serde::{Serialize, Deserialize};
use std::sync::Arc;
use super::{TradeAsset, ClassInfo};
use crate::{serialize::string, types::{AppId, ContextId, AssetId, Amount, ClassInfoClass}};

/// An asset which includes its related [`ClassInfo`] mapping.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Asset {
    /// The app ID e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    pub appid: AppId,
    #[serde(with = "string")]
    /// The context ID.
    pub contextid: ContextId,
    #[serde(with = "string")]
    /// The unique asset ID. This value is unique to the item's `appid` and `contextid`.
    pub assetid: AssetId,
    #[serde(with = "string")]
    /// The amount. If this item is not stackable the amount will be `1`.
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
/// `new_contextid` and `new_assetid` respectively.
impl From<TradeAsset> for Asset {
    fn from(trade_asset: TradeAsset) -> Self {
        Asset {
            appid: trade_asset.appid,
            contextid: trade_asset.new_contextid,
            assetid: trade_asset.new_assetid,
            amount: trade_asset.amount,
            classinfo: trade_asset.classinfo,
        }
    }
}

/// Converts a borrowed [`TradeAsset`] into an [`Asset`]. The `contextid` and `assetid` are taken 
/// from`new_contextid` and `new_assetid` respectively.
impl From<&TradeAsset> for Asset {
    fn from(trade_asset: &TradeAsset) -> Self {
        Asset {
            appid: trade_asset.appid,
            contextid: trade_asset.new_contextid,
            assetid: trade_asset.new_assetid,
            amount: trade_asset.amount,
            classinfo: Arc::clone(&trade_asset.classinfo),
        }
    }
}