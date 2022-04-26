use serde::{Serialize, Deserialize};
use std::sync::Arc;
use super::classinfo::ClassInfo;
use crate::types::{
    AppId,
    ContextId,
    AssetId,
    Amount,
    ClassInfoClass
};

/// An asset which includes its related [`ClassInfo`] mapping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub appid: AppId,
    pub contextid: ContextId,
    pub assetid: AssetId,
    pub amount: Amount,
    pub classinfo: Arc<ClassInfo>,
}

impl Asset {
    
    pub fn key(&self) -> ClassInfoClass {
        (self.appid, self.classinfo.classid, self.classinfo.instanceid)
    }
}