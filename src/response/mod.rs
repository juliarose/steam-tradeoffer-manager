pub mod trade_offer;
pub mod sent_offer;
pub mod classinfo;
pub mod asset;
pub mod user_details;
pub mod accepted_offer;
pub mod deserializers;

// pub use accepted_offer::AcceptedOffer;
// pub use user_details::UserDetails;
// pub use asset::Asset;
// pub use trade_offer::TradeOffer;
// pub use sent_offer::SentOffer;
// pub use classinfo::{
//     ClassInfo,
//     Action,
//     Description,
//     Tag
// };




// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::env;
//     use std::fs;
//     use std::path::Path;
//     use serde::de::DeserializeOwned;

//     fn read_file(filename: &str) -> std::io::Result<String> {
//         let rootdir = env!("CARGO_MANIFEST_DIR");
//         let filepath = Path::new(rootdir).join(format!("tests/json/{}", filename));
        
//         fs::read_to_string(filepath)
//     }
    
//     fn read_and_parse_file<D>(filename: &str) -> Result<D, &str>
//     where
//         D: DeserializeOwned
//     {
//         let contents = tests::read_file(filename)
//             .expect("Something went wrong reading the file");
//         let response: D = serde_json::from_str(&contents).unwrap();
        
//         Ok(response)
//     }
    
//     #[test]
//     fn parses_get_asset_classinfo_response() {
//         let response: GetAssetClassInfoResponse = tests::read_and_parse_file("get_asset_classinfo.json").unwrap();
//         let classinfo_string = response.result.get(&(101785959, Some(11040578))).unwrap();
//         let parsed = serde_json::from_str::<response::classinfo::ClassInfo>(classinfo_string).unwrap();

//         assert_eq!(parsed.market_hash_name, String::from("Mann Co. Supply Crate Key"));
//     }
    
//     #[test]
//     fn parses_get_trade_offers_response() {
//         let response: GetTradeOffersResponse = tests::read_and_parse_file("get_trade_offers.json").unwrap();
//         let offer = response.response.trade_offers_sent.first().unwrap();

//         assert_eq!(offer.message, Some(String::from("give me that key")));
//     }
    
//     #[test]
//     fn parses_get_inventory_response() {
//         let response: GetInventoryResponse = tests::read_and_parse_file("inventory.json").unwrap();
//         let asset = response.assets.first().unwrap();

//         assert_eq!(asset.assetid, 11152148507);
//     }
// }