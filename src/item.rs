use serde::{Serialize, Deserialize};
use crate::serializers::string;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Item {
    pub appid: u32,
    #[serde(with = "string")]
    pub contextid: u32,
    #[serde(with = "string")]
    pub assetid: u64,
    pub amount: u32,
}