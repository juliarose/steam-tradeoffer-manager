use std::sync::Arc;
use serde::{Serialize, Deserialize};
use steamid_ng::SteamID;
use chrono::serde::ts_seconds;
use crate::{
    response,
    ServerTime,
    error::MissingClassInfoError,
    enums::{TradeStatus, ConfirmationMethod, TradeOfferState},
    serializers::{
        string,
        option_string,
        option_string_0_as_none,
    },
    deserializers::{
        ts_seconds_option_none_when_zero,
        empty_string_is_none,
    },
    types::{
        AppId,
        ContextId,
        AssetId,
        Amount,
        ClassId,
        InstanceId,
        TradeOfferId,
        TradeId,
        ClassInfoMap,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawTradeOffer {
    #[serde(with = "string")]
    /// The ID for this offer.
    pub tradeofferid: TradeOfferId,
    #[serde(default)]
    #[serde(with = "option_string")]
    /// The trade ID for this offer. This should be present when the state of the offer is
    /// "Accepted".
    pub tradeid: Option<TradeId>,
    /// The [`SteamID`] of our partner.
    pub accountid_other: u32,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_is_none")]
    /// The message included in the offer. If the message is empty or not present this will
    /// be `None`.
    pub message: Option<String>,
    #[serde(default)]
    /// The items we're receiving in this offer.
    pub items_to_receive: Vec<RawAsset>,
    #[serde(default)]
    /// The items we're giving in this offer.
    pub items_to_give: Vec<RawAsset>,
    #[serde(default)]
    /// Whether this offer was created by us or not.
    pub is_our_offer: bool,
    #[serde(default)]
    /// Whether this offer originated from a real time trade.
    pub from_real_time_trade: bool,
    #[serde(with = "ts_seconds")]
    /// The time before the offer expires if it has not been acted on.
    pub expiration_time: ServerTime,
    #[serde(with = "ts_seconds")]
    /// The time this offer was created.
    pub time_created: ServerTime,
    #[serde(with = "ts_seconds")]
    /// The time this offer last had an action e.g. accepting or declining the offer.
    pub time_updated: ServerTime,
    /// The state of this offer.
    pub trade_offer_state: TradeOfferState,
    #[serde(with = "ts_seconds_option_none_when_zero")]
    /// The end date if this trade is in escrow. `None` when this offer is not in escrow.
    pub escrow_end_date: Option<ServerTime>,
    /// The confirmation method for this offer.
    pub confirmation_method: ConfirmationMethod,
}

impl RawTradeOffer {
    /// Attempts to combine this [`RawTradeOffer`] into a [`response::trade_offer::TradeOffer`] using the given map.
    pub fn try_combine_classinfos(
        self,
        map: &ClassInfoMap,
    ) -> Result<response::TradeOffer, MissingClassInfoError> {
        fn collect_items(
            assets: Vec<RawAsset>,
            map: &ClassInfoMap,
        ) -> Result<Vec<response::Asset>, MissingClassInfoError> {
            assets
                .into_iter()
                .map(|asset| {
                    if let Some(classinfo) = map.get(&(asset.appid, asset.classid, asset.instanceid)) {
                        Ok(response::Asset {
                            classinfo: Arc::clone(classinfo),
                            appid: asset.appid,
                            contextid: asset.contextid,
                            assetid: asset.assetid,
                            amount: asset.amount,
                        })
                    } else {
                        // todo use a less broad error for this
                        Err(MissingClassInfoError {
                            appid: asset.appid,
                            classid: asset.classid,
                            instanceid: asset.instanceid,
                        })
                    }
                })
                .collect()
        }
        
        Ok(response::TradeOffer {
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
    
    /// Whether the state of this offer can be modified. This is either active offers or offers 
    /// that are in escrow.
    pub fn state_is_changeable(&self) -> bool {
        self.trade_offer_state == TradeOfferState::Active ||
        self.trade_offer_state == TradeOfferState::InEscrow ||
        self.trade_offer_state == TradeOfferState::CreatedNeedsConfirmation
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawAsset {
    /// The appid e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    pub appid: AppId,
    #[serde(with = "string")]
    /// The context id.
    pub contextid: ContextId,
    #[serde(with = "string")]
    /// The unique asset ID. This value is unique to the item's appid and contextid.
    pub assetid: AssetId,
    #[serde(with = "string")]
    /// The ID of the classinfo.
    pub classid: ClassId,
    #[serde(with = "option_string_0_as_none")]
    /// The specific instance ID of the classinfo.
    pub instanceid: InstanceId,
    #[serde(with = "string")]
    /// The amount. If this item is not stackable the amount will be 1.
    pub amount: Amount,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawReceiptAsset {
    /// The appid e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    pub appid: AppId,
    /// The context id.
    pub contextid: ContextId,
    #[serde(with = "string", rename = "id")]
    /// The unique asset ID. This value is unique to the item's appid and contextid.
    pub assetid: AssetId,
    #[serde(with = "string")]
    /// The ID of the classinfo.
    pub classid: ClassId,
    #[serde(with = "option_string_0_as_none")]
    /// The specific instance ID of the classinfo.
    pub instanceid: InstanceId,
    #[serde(with = "string")]
    /// The amount. If this item is not stackable the amount will be 1.
    pub amount: Amount,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawAssetOld {
    #[serde(with = "string", rename = "id")]
    /// The unique asset ID.
    pub assetid: AssetId,
    #[serde(with = "string")]
    /// The ID of the classinfo.
    pub classid: ClassId,
    #[serde(with = "option_string_0_as_none")]
    /// The specific instance ID of the classinfo.
    pub instanceid: InstanceId,
    #[serde(with = "string")]
    /// The amount. If this item is not stackable the amount will be 1.
    pub amount: Amount,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RawTrade {
    #[serde(with = "string")]
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
    pub assets_given: Vec<RawTradeAsset>,
    #[serde(default)]
    /// Assets given.
    pub assets_received: Vec<RawTradeAsset>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RawTradeAsset {
    /// The appid e.g. 440 for Team Fortress 2 or 730 for Counter-Strike Global offensive.
    pub appid: AppId,
    #[serde(with = "string")]
    /// The context id.
    pub contextid: ContextId,
    #[serde(with = "string")]
    /// The unique asset ID. This value is unique to the item's appid and contextid.
    pub assetid: AssetId,
    #[serde(with = "string")]
    /// The ID of the classinfo.
    pub classid: ClassId,
    #[serde(with = "option_string_0_as_none")]
    /// The specific instance ID of the classinfo.
    pub instanceid: InstanceId,
    #[serde(with = "string")]
    /// The amount. If this item is not stackable the amount will be 1.
    pub amount: Amount,
    #[serde(with = "string")]
    /// The context id of the item received.
    pub new_contextid: ContextId,
    #[serde(with = "string")]
    /// The unique asset ID of the item received. This value is unique to the item's appid and contextid.
    pub new_assetid: AssetId,
}

impl RawTrade {
    /// Attempts to combine this [`RawTradeOffer`] into a [`response::trade_offer::TradeOffer`] using the given map.
    pub fn try_combine_classinfos(
        self,
        map: &ClassInfoMap,
    ) -> Result<response::Trade, MissingClassInfoError> {
        fn collect_items(
            assets: Vec<RawTradeAsset>,
            map: &ClassInfoMap,
        ) -> Result<Vec<response::TradeAsset>, MissingClassInfoError> {
            assets
                .into_iter()
                .map(|asset| {
                    if let Some(classinfo) = map.get(&(asset.appid, asset.classid, asset.instanceid)) {
                        Ok(response::TradeAsset {
                            classinfo: Arc::clone(classinfo),
                            appid: asset.appid,
                            contextid: asset.contextid,
                            assetid: asset.assetid,
                            amount: asset.amount,
                            new_contextid: asset.new_contextid,
                            new_assetid: asset.new_assetid,
                        })
                    } else {
                        // todo use a less broad error for this
                        Err(MissingClassInfoError {
                            appid: asset.appid,
                            classid: asset.classid,
                            instanceid: asset.instanceid,
                        })
                    }
                })
                .collect()
        }
        
        Ok(response::Trade {
            assets_given: collect_items(self.assets_given, map)?,
            assets_received: collect_items(self.assets_received, map)?,
            tradeid: self.tradeid,
            status: self.status,
            steamid_other: self.steamid_other,
            time_init: self.time_init,
        })
    }
}