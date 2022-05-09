use std::{
    collections::HashMap,
    sync::Arc,
    fmt,
};
use crate::{
    types::ClassInfoAppClass,
    response::{
        self,
        deserializers::{
            from_int_to_bool,
            to_classinfo_map,
            option_str_to_number,
            deserialize_classinfo_map_raw,
            deserialize_classinfo_map,
        }
    }
};
use super::raw;
use serde::{
    Deserialize,
    de::{
        MapAccess,
        Visitor,
        SeqAccess,
        Deserializer,
    },
};

type RgInventory = HashMap<String, raw::RawAssetOld>;

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
                let asset = access.next_value::<raw::RawAssetOld>()?;
                
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
    pub trade_offers_sent: Vec<raw::RawTradeOffer>,
    #[serde(default)]
    pub trade_offers_received: Vec<raw::RawTradeOffer>,
    pub next_cursor: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct GetTradeOffersResponse {
    pub response: GetTradeOffersResponseBody,
}

#[derive(Deserialize, Debug)]
pub struct GetInventoryResponse {
    #[serde(default)]
    #[serde(deserialize_with = "from_int_to_bool")]
    pub success: bool,
    #[serde(default)]
    #[serde(deserialize_with = "from_int_to_bool")]
    pub more_items: bool,
    #[serde(default)]
    pub assets: Vec<raw::RawAsset>,
    #[serde(deserialize_with = "to_classinfo_map")]
    pub descriptions: HashMap<ClassInfoAppClass, Arc<response::classinfo::ClassInfo>>,
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
    pub descriptions: HashMap<ClassInfoAppClass, Arc<response::classinfo::ClassInfo>>,
}

#[derive(Deserialize, Debug)]
pub struct GetAssetClassInfoResponse {
    #[serde(deserialize_with = "deserialize_classinfo_map_raw")]
    pub result: HashMap<ClassInfoAppClass, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parses_get_asset_classinfo_response() {
        let response: GetAssetClassInfoResponse = serde_json::from_str(include_str!("fixtures/get_asset_classinfo.json")).unwrap();
        let classinfo_string = response.result.get(&(101785959, Some(11040578))).unwrap();
        let parsed = serde_json::from_str::<response::classinfo::ClassInfo>(classinfo_string).unwrap();

        assert_eq!(parsed.market_hash_name, String::from("Mann Co. Supply Crate Key"));
    }
    
    #[test]
    fn parses_get_trade_offers_response() {
        let response: GetTradeOffersResponse = serde_json::from_str(include_str!("fixtures/get_trade_offers.json")).unwrap();
        let offer = response.response.trade_offers_sent.first().unwrap();

        assert_eq!(offer.message, Some(String::from("give me that key")));
    }
    
    #[test]
    fn parses_get_inventory_response() {
        let response: GetInventoryResponse = serde_json::from_str(include_str!("fixtures/inventory.json")).unwrap();
        let asset = response.assets.first().unwrap();

        assert_eq!(asset.assetid, 11152148507);
    }
}