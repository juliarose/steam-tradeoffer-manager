use super::ClassInfo;
use std::sync::Arc;
use deepsize::DeepSizeOf;

#[derive(DeepSizeOf, Debug)]
pub struct Asset {
    pub appid: u32,
    pub contextid: u32,
    pub assetid: u64,
    pub amount: u32,
    pub classinfo: Arc<ClassInfo>,
}
