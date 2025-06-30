//! Includes raw response models for API responses.

use crate::types::*;
use crate::error::{MissingClassInfoError, TryIntoNewAssetError};
use crate::response::{TradeOffer, Asset, Trade, TradeAsset};
use crate::enums::{TradeStatus, ConfirmationMethod, TradeOfferState};
use crate::serialize;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use steamid_ng::SteamID;
use chrono::serde::ts_seconds;

/// Trade offer.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawTradeOffer {
    /// The ID for this offer.
    #[serde(with = "serialize::string")]
    pub tradeofferid: TradeOfferId,
    /// The trade ID for this offer. This should be present when the `trade_offer_state` of this 
    /// offer is [`TradeOfferState::Accepted`]. It can also be present if the offer was accepted 
    /// but the trade is not yet complete. The trade should appear in your trade history.
    #[serde(default)]
    #[serde(with = "serialize::option_string")]
    pub tradeid: Option<TradeId>,
    /// The [`SteamID`] of our partner.
    pub accountid_other: u32,
    /// The message included in the offer. If the message is empty or not present this will be 
    /// `None`.
    #[serde(default)]
    #[serde(deserialize_with = "serialize::empty_string_is_none")]
    pub message: Option<String>,
    /// The items we're receiving in this offer.
    #[serde(default)]
    pub items_to_receive: Vec<RawAsset>,
    /// The items we're giving in this offer.
    #[serde(default)]
    pub items_to_give: Vec<RawAsset>,
    /// Whether this offer was created by us or not.
    #[serde(default)]
    pub is_our_offer: bool,
    /// Whether this offer originated from a real time trade.
    #[serde(default)]
    pub from_real_time_trade: bool,
    /// The time before the offer expires if it has not been acted on.
    #[serde(with = "ts_seconds")]
    pub expiration_time: ServerTime,
    /// The time this offer was created.
    #[serde(with = "ts_seconds")]
    pub time_created: ServerTime,
    /// The time this offer last had an action e.g. accepting or declining the offer.
    #[serde(with = "ts_seconds")]
    pub time_updated: ServerTime,
    /// The state of this offer.
    pub trade_offer_state: TradeOfferState,
    /// The end date if this trade is in escrow. `None` when this offer is not in escrow.
    #[serde(with = "serialize::ts_seconds_option_none_when_zero")]
    pub escrow_end_date: Option<ServerTime>,
    /// The confirmation method for this offer.
    pub confirmation_method: ConfirmationMethod,
}

impl RawTradeOffer {
    /// Attempts to combine this [`RawTradeOffer`] into a [`TradeOffer`] using the given map.
    pub fn try_combine_classinfos(
        self,
        map: &ClassInfoMap,
    ) -> Result<TradeOffer, MissingClassInfoError> {
        fn collect_items(
            assets: Vec<RawAsset>,
            map: &ClassInfoMap,
        ) -> Result<Vec<Asset>, MissingClassInfoError> {
            assets
                .into_iter()
                .map(|asset| {
                    if let Some(classinfo) = map.get(&(asset.appid, asset.classid, asset.instanceid)) {
                        Ok(Asset {
                            appid: asset.appid,
                            contextid: asset.contextid,
                            assetid: asset.assetid,
                            amount: asset.amount,
                            missing: asset.missing,
                            classinfo: Arc::clone(classinfo),
                        })
                    } else {
                        Err(MissingClassInfoError {
                            appid: asset.appid,
                            classid: asset.classid,
                            instanceid: asset.instanceid,
                        })
                    }
                })
                .collect()
        }
        
        Ok(TradeOffer {
            items_to_give: collect_items(self.items_to_give, map)?,
            items_to_receive: collect_items(self.items_to_receive, map)?,
            tradeofferid: self.tradeofferid,
            tradeid: self.tradeid,
            trade_offer_state: self.trade_offer_state,
            partner: SteamID::new(
                self.accountid_other,
                steamid_ng::Instance::Desktop,
                steamid_ng::AccountType::Individual,
                steamid_ng::Universe::Public
            ),
            message: self.message,
            is_our_offer: self.is_our_offer,
            from_real_time_trade: self.from_real_time_trade,
            expiration_time: self.expiration_time,
            time_updated: self.time_updated,
            time_created: self.time_created,
            escrow_end_date: self.escrow_end_date,
            confirmation_method: self.confirmation_method,
        })
    }
    
    /// Checks whether the trade offer is glitched or not by checking if no items are present.
    pub fn is_glitched(&self) -> bool {
        self.items_to_receive.is_empty() && self.items_to_give.is_empty()
    }
}

/// An asset belonging to a [`RawTrade`].
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RawAsset {
    /// The app ID e.g. `440` for Team Fortress 2 or `730` for Counter-Strike Global Offensive.
    pub appid: AppId,
    /// The context ID.
    #[serde(with = "serialize::string")]
    pub contextid: ContextId,
    /// The unique asset ID. This value is unique to the item's `appid` and `contextid`.
    #[serde(with = "serialize::string")]
    pub assetid: AssetId,
    /// The amount. If this item is not stackable the amount will be `1`.
    #[serde(with = "serialize::string")]
    pub amount: Amount,
    /// `true` if the item no longer exists in the inventory.
    #[serde(default)]
    pub missing: bool,
    /// The ID of the classinfo.
    #[serde(with = "serialize::string")]
    pub classid: ClassId,
    /// The specific instance ID of the classinfo belonging to the class ID.
    #[serde(with = "serialize::option_string_0_as_none")]
    pub instanceid: InstanceId,
}

/// Converts a [`RawTradeAsset`] into a [`RawAsset`]. The `contextid` and `assetid` are taken from 
/// `contextid` and `assetid` respectively, **not** `new_contextid` and `new_assetid`.
/// 
/// If you need a [`RawAsset`] of the newly acquired item, call `try_into_new_asset` on the
/// [`RawTradeAsset`].
impl From<RawTradeAsset> for RawAsset {
    fn from(raw_trade_asset: RawTradeAsset) -> Self {
        Self {
            appid: raw_trade_asset.appid,
            contextid: raw_trade_asset.contextid,
            assetid: raw_trade_asset.assetid,
            amount: raw_trade_asset.amount,
            missing: false,
            classid: raw_trade_asset.classid,
            instanceid: raw_trade_asset.instanceid,
        }
    }
}

/// Converts a borrowed [`RawTradeAsset`] into a [`RawAsset`]. The `contextid` and `assetid` are 
/// taken from `contextid` and `assetid` respectively, **not** `new_contextid` and `new_assetid`.
/// 
/// If you need a [`RawAsset`] of the newly acquired item, call `try_into_new_asset` on the
/// [`RawTradeAsset`].
impl From<&RawTradeAsset> for RawAsset {
    fn from(raw_trade_asset: &RawTradeAsset) -> Self {
        Self {
            appid: raw_trade_asset.appid,
            contextid: raw_trade_asset.contextid,
            assetid: raw_trade_asset.assetid,
            amount: raw_trade_asset.amount,
            missing: false,
            classid: raw_trade_asset.classid,
            instanceid: raw_trade_asset.instanceid,
        }
    }
}

/// An asset belonging to a receipt.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RawReceiptAsset {
    /// The app ID e.g. `440` for Team Fortress 2 or `730` for Counter-Strike Global Offensive.
    pub appid: AppId,
    /// The context ID.
    pub contextid: ContextId,
    /// The unique asset ID. This value is unique to the item's `appid` and `contextid`.
    #[serde(with = "serialize::string", rename = "id")]
    pub assetid: AssetId,
    /// The amount. If this item is not stackable the amount will be `1`.
    #[serde(with = "serialize::string")]
    pub amount: Amount,
    /// The ID of the classinfo.
    #[serde(with = "serialize::string")]
    pub classid: ClassId,
    /// The specific instance ID of the classinfo belonging to the class ID.
    #[serde(with = "serialize::option_string_0_as_none")]
    pub instanceid: InstanceId,
}

/// An asset from the old inventory API.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RawAssetOld {
    /// The unique asset ID.
    #[serde(with = "serialize::string", rename = "id")]
    pub assetid: AssetId,
    /// The amount. If this item is not stackable the amount will be `1`.
    #[serde(with = "serialize::string")]
    pub amount: Amount,
    /// The ID of the classinfo.
    #[serde(with = "serialize::string")]
    pub classid: ClassId,
    /// The specific instance ID of the classinfo belonging to the class ID.
    #[serde(with = "serialize::option_string_0_as_none")]
    pub instanceid: InstanceId,
}

/// Details from a GetTradeHistory response.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawTrades {
    /// The trades.
    pub trades: Vec<RawTrade>,
    /// Whether more trades can be fetched.
    pub more: bool,
    /// The total trades of your account.
    pub total_trades: u32,
}

/// Trade.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawTrade {
    /// The trade ID.
    #[serde(with = "serialize::string")]
    pub tradeid: TradeId,
    /// The [`SteamID`] of our partner.
    pub steamid_other: SteamID,
    /// The time the trade was initiated.
    #[serde(with = "ts_seconds")]
    pub time_init: ServerTime,
    /// The current status of the trade.
    pub status: TradeStatus,
    /// Assets given.
    #[serde(default)]
    pub assets_given: Vec<RawTradeAsset>,
    /// Assets given.
    #[serde(default)]
    pub assets_received: Vec<RawTradeAsset>,
}

/// An asset belonging to a [`RawTrade`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawTradeAsset {
    /// The app ID e.g. `440` for Team Fortress 2 or `730` for Counter-Strike Global Offensive.
    pub appid: AppId,
    /// The context ID.
    #[serde(with = "serialize::string")]
    pub contextid: ContextId,
    /// The unique asset ID. This value is unique to the item's `appid` and `contextid`.
    #[serde(with = "serialize::string")]
    pub assetid: AssetId,
    /// The amount. If this item is not stackable the amount will be `1`.
    #[serde(with = "serialize::string")]
    pub amount: Amount,
    /// The ID of the classinfo.
    #[serde(with = "serialize::string")]
    pub classid: ClassId,
    /// The specific instance ID of the classinfo belonging to the class ID.
    #[serde(with = "serialize::option_string_0_as_none")]
    pub instanceid: InstanceId,
    /// The context ID of the item received. `None` if this item has not yet finished 
    /// transferring.
    #[serde(default)]
    #[serde(with = "serialize::option_string")]
    pub new_contextid: Option<ContextId>,
    /// The unique asset ID of the item received. `None` if this item has not yet finished 
    /// transferring.
    #[serde(default)]
    #[serde(with = "serialize::option_string")]
    pub new_assetid: Option<AssetId>,
}

impl RawTradeAsset {
    /// Attempts to convert this [`TradeAsset`] into an [`Asset`] of the newly acquired item. The 
    /// `contextid` and `assetid` are taken from `new_contextid` and `new_assetid` respectively.
    /// 
    /// Fails if the `new_contextid` and `new_assetid` properties are not present. This occurs 
    /// during trades that have either failed or have yet to complete and the item has not been
    /// transferred. Check that the `trade_status` of the [`Trade`] this asset belongs to is 
    /// [`crate::enums::TradeStatus::Complete`].
    pub fn try_into_new_asset(&self) -> Result<RawAsset, TryIntoNewAssetError> {
        let contextid = self.new_contextid
            .ok_or(TryIntoNewAssetError {
                appid: self.appid,
                contextid: self.contextid,
                assetid: self.assetid,
                amount: self.amount,
            })?;
        let assetid = self.new_assetid
            .ok_or(TryIntoNewAssetError {
                appid: self.appid,
                contextid: self.contextid,
                assetid: self.assetid,
                amount: self.amount,
            })?;
        
        Ok(RawAsset {
            appid: self.appid,
            contextid,
            assetid,
            amount: self.amount,
            missing: false,
            classid: self.classid,
            instanceid: self.instanceid,
        })
    }
}

impl RawTrade {
    /// Attempts to combine this [`RawTradeOffer`] into a [`Trade`] using the given map.
    pub fn try_combine_classinfos(
        self,
        map: &ClassInfoMap,
    ) -> Result<Trade, MissingClassInfoError> {
        fn collect_items(
            assets: Vec<RawTradeAsset>,
            map: &ClassInfoMap,
        ) -> Result<Vec<TradeAsset>, MissingClassInfoError> {
            assets
                .into_iter()
                .map(|asset| {
                    if let Some(classinfo) = map.get(&(asset.appid, asset.classid, asset.instanceid)) {
                        Ok(TradeAsset {
                            appid: asset.appid,
                            contextid: asset.contextid,
                            assetid: asset.assetid,
                            amount: asset.amount,
                            new_contextid: asset.new_contextid,
                            new_assetid: asset.new_assetid,
                            classinfo: Arc::clone(classinfo),
                        })
                    } else {
                        Err(MissingClassInfoError {
                            appid: asset.appid,
                            classid: asset.classid,
                            instanceid: asset.instanceid,
                        })
                    }
                })
                .collect()
        }
        
        Ok(Trade {
            assets_given: collect_items(self.assets_given, map)?,
            assets_received: collect_items(self.assets_received, map)?,
            tradeid: self.tradeid,
            status: self.status,
            steamid_other: self.steamid_other,
            time_init: self.time_init,
        })
    }
}
