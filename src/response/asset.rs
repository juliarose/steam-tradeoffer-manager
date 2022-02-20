use serde::{Serialize, Deserialize};
use std::sync::Arc;
use super::classinfo::ClassInfo;
use crate::types::{
    AppId,
    ContextId,
    AssetId,
    Amount
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub appid: AppId,
    pub contextid: ContextId,
    pub assetid: AssetId,
    pub amount: Amount,
    pub classinfo: Arc<ClassInfo>,
}
