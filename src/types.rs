//! Types for common values in Steam responses.

/// Uniquely identifies an application on Steam. For example: 440 for Team Fortress 2.
pub type AppId = u32;
/// A context ID belonging to an [`AppId`].
pub type ContextId = u64;
/// An asset ID unique to an [`AppId`] + [`ContextId`] combination.
pub type AssetId = u64;
/// An amount for stackable items. For non-stackable items this is simply `1`.
pub type Amount = u32;
/// An ID for a [`ClassInfo`] which provides a general overview of an item.
pub type ClassId = u64;
/// A more specific instance of a [`ClassInfo`], for example a Team Fortress 2 item which is 
/// painted.
pub type InstanceId = Option<u64>;
/// An ID of a trade offer.
pub type TradeOfferId = u64;
/// An ID of a trade.
pub type TradeId = u128;


pub use crate::time::ServerTime;

// Types internally used by the crate.
use crate::response::ClassInfo;
use std::sync::Arc;
use std::collections::HashMap;
use reqwest_middleware::ClientWithMiddleware;

pub(crate) type HttpClient = ClientWithMiddleware;
pub(crate) type ClassInfoClass = (AppId, ClassId, InstanceId);
pub(crate) type ClassInfoMap = HashMap<ClassInfoClass, Arc<ClassInfo>>;
pub(crate) type ClassInfoAppClass = (ClassId, InstanceId);
pub(crate) type ClassInfoAppMap = HashMap<ClassInfoAppClass, Arc<ClassInfo>>;