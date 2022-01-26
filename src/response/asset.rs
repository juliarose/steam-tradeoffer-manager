use std::sync::Arc;
use super::ClassInfo;
use crate::types::{
    AppId,
    ContextId,
    AssetId,
    Amount
};

#[derive(Debug)]
pub struct Asset {
    pub appid: AppId,
    pub contextid: ContextId,
    pub assetid: AssetId,
    pub amount: Amount,
    pub classinfo: Arc<ClassInfo>,
}
