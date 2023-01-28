use serde::{Serialize, Deserialize};
use std::sync::Arc;
use super::classinfo::ClassInfo;
use crate::{
    serializers::string,
    types::{AppId, ContextId, AssetId, Amount, ClassInfoClass},
};

/// An asset which includes its related [`ClassInfo`] mapping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    /// The appid e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    pub appid: AppId,
    #[serde(with = "string")]
    /// The context id.
    pub contextid: ContextId,
    #[serde(with = "string")]
    /// The unique asset ID. This value is unique to the item's appid and contextid.
    pub assetid: AssetId,
    #[serde(with = "string")]
    /// The amount. If this item is not stackable the amount will be 1.
    pub amount: Amount,
    /// The [`ClassInfo`] containing names, descriptions and other details about the item.
    pub classinfo: Arc<ClassInfo>,
}

impl Asset {
    /// The key used for [`ClassInfo`] data.
    pub fn key(&self) -> ClassInfoClass {
        (self.appid, self.classinfo.classid, self.classinfo.instanceid)
    }
}