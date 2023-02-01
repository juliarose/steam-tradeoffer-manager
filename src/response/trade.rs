use crate::{
    SteamID,
    time::ServerTime,
    response::ClassInfo,
    enums::TradeStatus,
    types::{TradeId, AppId, ContextId, AssetId, Amount},
    error::TryIntoNewAssetError,
    serialize::{string, option_string},
};
use chrono::serde::ts_seconds;
use serde::{self, Deserialize, Serialize};
use std::sync::Arc;
use super::Asset;

/// Details from a GetTradeHistory response.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trades {
    /// The trades.
    pub trades: Vec<Trade>,
    /// Whether more trades can be fetched.
    pub more: bool,
    /// The total trades of your account.
    pub total_trades: u32,
}

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
    /// The context ID of the item received. `None` if this item has not yet finished transferring.
    #[serde(with = "option_string")]
    pub new_contextid: Option<ContextId>,
    /// The unique asset ID of the item received. `None` if this item has not yet finished t
    /// ransferring.
    #[serde(with = "option_string")]
    pub new_assetid: Option<AssetId>,
    /// The [`ClassInfo`] containing names, descriptions, and other details about the item.
    pub classinfo: Arc<ClassInfo>,
}

impl TradeAsset {
    /// Attempts to convert this [`TradeAsset`] into an [`Asset`] of the newly acquired item. The 
    /// `contextid` and `assetid` are taken from `new_contextid` and `new_assetid` respectively.
    /// 
    /// Fails if the `new_contextid` and `new_assetid` properties are not present. This occurs 
    /// during trades that have either failed or have yet to complete and the item has not been
    /// transferred. Check that the `trade_status` of the [`Trade`] this asset belongs to is 
    /// [`crate::enums::TradeStatus::Complete`].
    pub fn try_into_new_asset(&self) -> Result<Asset, TryIntoNewAssetError> {
        let contextid = self.new_contextid
            .ok_or_else(|| TryIntoNewAssetError {
                appid: self.appid,
                contextid: self.contextid,
                assetid: self.assetid,
                amount: self.amount,
            })?;
        let assetid = self.new_assetid
            .ok_or_else(|| TryIntoNewAssetError {
                appid: self.appid,
                contextid: self.contextid,
                assetid: self.assetid,
                amount: self.amount,
            })?;
        
        Ok(Asset {
            appid: self.appid,
            contextid,
            assetid,
            amount: self.amount,
            classinfo: Arc::clone(&self.classinfo),
        })
    }
}