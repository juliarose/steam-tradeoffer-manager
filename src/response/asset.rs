use super::{ClassInfo, TradeAsset};
use crate::serialize;
use crate::types::{Amount, AppId, AssetId, ClassInfoClass, ContextId};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// An asset which includes its related [`ClassInfo`] mapping.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Asset {
    /// The app ID e.g. `440` for Team Fortress 2 or `730` for Counter-Strike Global Offensive.
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
    /// Properties of the asset, if available.
    #[serde(default)]
    pub properties: Option<Vec<AssetProperty>>,
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
            properties: None,
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
            properties: None,
        }
    }
}

/// Value of an asset property.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AssetPropertyValue {
    /// Integer value.
    Int(i32),
    /// Floating point value.
    Float(f32),
}

/// Properties of an asset.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AssetProperty {
    /// ID of the property. This may be `None` if the property does not have an ID.
    pub propertyid: i32,
    /// Name of the property.
    pub name: String,
    /// Value of the property.
    pub value: AssetPropertyValue,
}
