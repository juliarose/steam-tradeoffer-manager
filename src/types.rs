//! Types for common values in Steam responses.

pub type AppId = u32;
pub type ContextId = u64;
pub type AssetId = u64;
pub type Amount = u32;
pub type ClassId = u64;
pub type InstanceId = Option<u64>;
pub type TradeOfferId = u64;
pub type TradeId = u128;

// Types internally used by the crate.
use crate::response::ClassInfo;
use std::sync::Arc;
use std::collections::HashMap;
use reqwest_middleware::ClientWithMiddleware;

pub(crate) type Client = ClientWithMiddleware;
pub(crate) type ClassInfoClass = (AppId, ClassId, InstanceId);
pub(crate) type ClassInfoMap = HashMap<ClassInfoClass, Arc<ClassInfo>>;
pub(crate) type ClassInfoAppClass = (ClassId, InstanceId);
pub(crate) type ClassInfoAppMap = HashMap<ClassInfoAppClass, Arc<ClassInfo>>;