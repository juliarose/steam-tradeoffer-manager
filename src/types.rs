use std::{sync::Arc, collections::HashMap};
use crate::response::{
    asset::Asset,
    classinfo::ClassInfo
};

pub type Inventory = Vec<Asset>;
pub type AppId = u32;
pub type ContextId = u32;
pub type AssetId = u64;
pub type Amount = u32;
pub type ClassId = u64;
pub type InstanceId = Option<u64>;
pub type TradeOfferId = u64;
pub type ClassInfoAppClass = (ClassId, InstanceId);
pub type ClassInfoClass = (AppId, ClassId, InstanceId);
pub type ClassInfoMap = HashMap<ClassInfoClass, Arc<ClassInfo>>;
pub type ClassInfoAppMap = HashMap<ClassInfoAppClass, Arc<ClassInfo>>;