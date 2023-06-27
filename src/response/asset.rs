use super::{TradeAsset, ClassInfo};
use crate::serialize;
use crate::types::{AppId, ContextId, AssetId, Amount};
use crate::internal_types::ClassInfoClass;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// An asset which includes its related [`ClassInfo`] mapping.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Asset {
    /// The app ID e.g. `440` for Team Fortress 2 or `730` for Counter-Strike Global offensive.
    pub appid: AppId,
    /// The context ID.
    #[serde(with = "serialize::string")]
    pub contextid: ContextId,
    /// The unique asset ID. This value is unique to the item's `appid` and `contextid`.
    #[serde(with = "serialize::string")]
    pub assetid: AssetId,
    /// The amount. If this item is not stackable the amount will be `1`.
    #[serde(with = "serialize::string")]
    pub amount: Amount,
    /// `true` if the item no longer exists in the inventory.
    #[serde(default)]
    pub missing: bool,
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
            missing: false,
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
            missing: false,
            classinfo: Arc::clone(&trade_asset.classinfo),
        }
    }
}