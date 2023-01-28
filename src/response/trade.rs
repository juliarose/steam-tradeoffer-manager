use crate::{
    SteamID,
    time::ServerTime,
    response::ClassInfo,
    enums::TradeStatus,
    types::{TradeId, AppId, ContextId, AssetId, Amount},
    serializers::string,
};
use chrono::serde::ts_seconds;
use serde::{self, Deserialize, Serialize};
use std::sync::Arc;

/// A trade.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub tradeid: TradeId,
    /// The [`SteamID`] of our partner.
    pub steamid_other: SteamID,
    #[serde(with = "ts_seconds")]
    /// The time the trade was initiated.
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

/// An asset belonging to a trade.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeAsset {
    #[serde(with = "string")]
    /// The appid e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    pub appid: AppId,
    /// The context id.
    pub contextid: ContextId,
    #[serde(with = "string")]
    /// The unique asset ID. This value is unique to the item's appid and contextid.
    pub assetid: AssetId,
    #[serde(with = "string")]
    /// The amount. If this item is not stackable the amount will be 1.
    pub amount: Amount,
    #[serde(with = "string")]
    /// The context id of the item received.
    pub new_contextid: ContextId,
    #[serde(with = "string")]
    /// The unique asset ID of the item received. This value is unique to the item's appid and contextid.
    pub new_assetid: AssetId,
    /// The [`ClassInfo`] containing names, descriptions and other details about the item.
    pub classinfo: Arc<ClassInfo>,
}