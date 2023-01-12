use std::sync::Arc;
use serde::{Serialize, Deserialize};
use chrono::serde::ts_seconds;
use steamid_ng::SteamID;
use crate::{
    response,
    ServerTime,
    error::MissingClassInfoError,
    enums::{ConfirmationMethod, TradeOfferState},
    serializers::{
        string,
        option_string,
        option_string_0_as_none
    },
    types::{
        AppId,
        ContextId,
        AssetId,
        Amount,
        ClassId,
        InstanceId,
        TradeOfferId,
        TradeId, ClassInfoMap
    },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct RawTradeOfferNoItems {
    #[serde(with = "string")]
    pub tradeofferid: TradeOfferId,
    #[serde(with = "option_string")]
    pub tradeid: Option<TradeId>,
    pub accountid_other: u32,
    pub message: Option<String>,
    #[serde(default)]
    pub is_our_offer: bool,
    #[serde(default)]
    pub from_real_time_trade: bool,
    #[serde(with = "ts_seconds")]
    pub expiration_time: ServerTime,
    #[serde(with = "ts_seconds")]
    pub time_created: ServerTime,
    #[serde(with = "ts_seconds")]
    pub time_updated: ServerTime,
    pub trade_offer_state: TradeOfferState,
    // todo parse 0 responses as null
    #[serde(with = "ts_seconds")]
    pub escrow_end_date: ServerTime,
    pub confirmation_method: ConfirmationMethod,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawTradeOffer {
    #[serde(with = "string")]
    pub tradeofferid: TradeOfferId,
    #[serde(default)]
    #[serde(with = "option_string")]
    pub tradeid: Option<TradeId>,
    pub accountid_other: u32,
    pub message: Option<String>,
    #[serde(default)]
    pub items_to_receive: Vec<RawAsset>,
    #[serde(default)]
    pub items_to_give: Vec<RawAsset>,
    #[serde(default)]
    pub is_our_offer: bool,
    #[serde(default)]
    pub from_real_time_trade: bool,
    #[serde(with = "ts_seconds")]
    pub expiration_time: ServerTime,
    #[serde(with = "ts_seconds")]
    pub time_created: ServerTime,
    #[serde(with = "ts_seconds")]
    pub time_updated: ServerTime,
    pub trade_offer_state: TradeOfferState,
    // todo parse 0 responses as null
    #[serde(with = "ts_seconds")]
    pub escrow_end_date: ServerTime,
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
                .collect::<Result<_, _>>()
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
    pub appid: AppId,
    #[serde(with = "string")]
    pub contextid: ContextId,
    #[serde(with = "string")]
    pub assetid: AssetId,
    #[serde(with = "string")]
    pub classid: ClassId,
    #[serde(with = "option_string_0_as_none")]
    pub instanceid: InstanceId,
    #[serde(with = "string")]
    pub amount: Amount,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawReceiptAsset {
    pub appid: AppId,
    pub contextid: ContextId,
    #[serde(with = "string", rename = "id")]
    pub assetid: AssetId,
    #[serde(with = "string")]
    pub classid: ClassId,
    #[serde(with = "option_string_0_as_none")]
    pub instanceid: InstanceId,
    #[serde(with = "string")]
    pub amount: Amount,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawAssetOld {
    #[serde(with = "string", rename = "id")]
    pub assetid: AssetId,
    #[serde(with = "string")]
    pub classid: ClassId,
    #[serde(with = "option_string_0_as_none")]
    pub instanceid: InstanceId,
    #[serde(with = "string")]
    pub amount: Amount,
}