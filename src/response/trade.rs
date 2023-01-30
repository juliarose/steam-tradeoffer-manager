use crate::{
    SteamID,
    time::ServerTime,
    response::ClassInfo,
    enums::TradeStatus,
    types::{TradeId, AppId, ContextId, AssetId, Amount},
    serialize::string,
};
use chrono::serde::ts_seconds;
use serde::{self, Deserialize, Serialize};
use std::sync::Arc;

/// A trade.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    /// The trade ID.
    pub tradeid: TradeId,
    /// The [`SteamID`] of our partner.
    pub steamid_other: SteamID,
    /// The time the trade was initiated.
    #[serde(with = "ts_seconds")]
    pub time_init: ServerTime,
    /// The current status of the trade.
    pub status: TradeStatus,
    #[serde(default)]
    /// Assets given.
    pub assets_given: Vec<TradeAsset>,
    #[serde(default)]
    /// Assets given.
    pub assets_received: Vec<TradeAsset>,
}

impl Default for Trade {
    fn default() -> Self {
        Trade {
            tradeid: 0,
            steamid_other: SteamID::from(0),
            time_init: chrono::Utc::now(),
            status: TradeStatus::Complete,
            assets_given: Vec::new(),
            assets_received: Vec::new(),
        }
    }
}

/// An asset belonging to a trade.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeAsset {
    /// The app ID e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    #[serde(with = "string")]
    pub appid: AppId,
    /// The context ID.
    pub contextid: ContextId,
    /// The unique asset ID. This value is unique to the item's `appid` and `contextid`.
    #[serde(with = "string")]
    pub assetid: AssetId,
    /// The amount. If this item is not stackable the amount will be `1`.
    #[serde(with = "string")]
    pub amount: Amount,
    /// The context ID of the item received.
    #[serde(with = "string")]
    pub new_contextid: ContextId,
    /// The unique asset ID of the item received. This value is unique to the item's `appid` and 
    /// `contextid`.
    #[serde(with = "string")]
    pub new_assetid: AssetId,
    /// The [`ClassInfo`] containing names, descriptions, and other details about the item.
    pub classinfo: Arc<ClassInfo>,
}