use serde::Deserialize;
use chrono::serde::ts_seconds;
use crate::{
    ConfirmationMethod,
    TradeOfferState,
    ServerTime,
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
        TradeId
    }
};

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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