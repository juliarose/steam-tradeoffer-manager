use std::{collections::HashMap, sync::Arc, fmt};
use crate::{
    response,
    internal_types::{ClassInfoAppClass, ClassInfoMap},
    serialize::{
        from_int_to_bool,
        to_trade_offers_classinfo_map,
        option_str_to_number,
        deserialize_classinfo_map_raw,
        deserialize_classinfo_map,
    },
};
use super::{response as api_response, RawTrade};
use serde::{Deserialize, de::{MapAccess, Visitor, SeqAccess, Deserializer}};

type RgInventory = HashMap<String, api_response::RawAssetOld>;

fn deserialize_rg_inventory<'de, D>(deserializer: D) -> Result<RgInventory, D::Error>
where
    D: Deserializer<'de>,
{
    struct RgInventoryVisitor;
    
    impl<'de> Visitor<'de> for RgInventoryVisitor {
        type Value = RgInventory;
        
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map or seq")
        }
    
        fn visit_seq<M>(self, mut _seq: M) -> Result<Self::Value, M::Error>
        where
            M: SeqAccess<'de>,
        {
            Ok(Self::Value::new())
        }
    
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = Self::Value::new();
            
            while let Some(key) = access.next_key::<String>()? {
                let asset = access.next_value::<api_response::RawAssetOld>()?;
                
                map.insert(key, asset);
            }
            
            Ok(map)
        }
    }
    
    deserializer.deserialize_any(RgInventoryVisitor)
}

#[derive(Deserialize, Debug)]
pub struct GetTradeOffersResponseBody {
    #[serde(default)]
    pub trade_offers_sent: Vec<api_response::RawTradeOffer>,
    #[serde(default)]
    pub trade_offers_received: Vec<api_response::RawTradeOffer>,
    #[serde(default)]
    #[serde(deserialize_with = "to_trade_offers_classinfo_map")]
    pub descriptions: Option<ClassInfoMap>,
    pub next_cursor: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct GetTradeOffersResponse {
    pub response: GetTradeOffersResponseBody,
}

// This ignores parsing the descriptions.
#[derive(Deserialize, Debug)]
pub struct GetInventoryResponseIgnoreDescriptions {
    #[serde(default)]
    #[serde(deserialize_with = "from_int_to_bool")]
    pub success: bool,
    #[serde(default)]
    #[serde(deserialize_with = "from_int_to_bool")]
    pub more_items: bool,
    #[serde(default)]
    pub assets: Vec<api_response::RawAsset>,
    #[serde(default)]
    #[serde(deserialize_with = "option_str_to_number")]
    pub last_assetid: Option<u64>,
    
}

#[derive(Deserialize, Debug)]
pub struct GetInventoryOldResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    #[serde(rename = "more")]
    pub more_items: bool,
    #[serde(default)]
    #[serde(deserialize_with = "option_str_to_number", rename = "more_start")]
    pub more_start: Option<u64>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_rg_inventory", rename = "rgInventory")]
    pub assets: RgInventory,
    #[serde(deserialize_with = "deserialize_classinfo_map", rename = "rgDescriptions")]
    pub descriptions: HashMap<ClassInfoAppClass, Arc<response::ClassInfo>>,
}

#[derive(Deserialize, Debug)]
pub struct GetAssetClassInfoResponse {
    #[serde(deserialize_with = "deserialize_classinfo_map_raw")]
    pub result: HashMap<ClassInfoAppClass, String>,
}

#[derive(Deserialize, Debug)]
pub struct GetTradeHistoryResponse {
    pub response: GetTradeHistoryResponseBody,
}

#[derive(Deserialize, Debug)]
pub struct GetTradeHistoryResponseBody {
    pub more: bool,
    pub total_trades: Option<u32>,
    pub trades: Vec<RawTrade>,
    #[serde(default)]
    #[serde(deserialize_with = "to_trade_offers_classinfo_map")]
    pub descriptions: Option<ClassInfoMap>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parses_get_asset_classinfo_response() {
        let response: GetAssetClassInfoResponse = serde_json::from_str(include_str!("fixtures/get_asset_classinfo.json")).unwrap();
        let classinfo_string = response.result.get(&(101785959, Some(11040578))).unwrap();
        let parsed = serde_json::from_str::<response::ClassInfo>(classinfo_string).unwrap();
        
        assert_eq!(parsed.market_hash_name, String::from("Mann Co. Supply Crate Key"));
    }
    
    #[test]
    fn parses_get_trade_offers_response() {
        let response: GetTradeOffersResponse = serde_json::from_str(include_str!("fixtures/get_trade_offers.json")).unwrap();
        let offer = response.response.trade_offers_sent.first().unwrap();
        
        assert!(offer.escrow_end_date.is_none());
        assert_eq!(offer.message, Some(String::from("give me that key")));
    }
}