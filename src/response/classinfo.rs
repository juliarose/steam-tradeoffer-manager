use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use super::deserializers::{
    into_bool,
    hashmap_or_vec,
    from_fraudwarnings,
    string_or_number
};
use crate::serializers::{
    string,
    option_string
};
use deepsize::DeepSizeOf;

#[derive(DeepSizeOf, Serialize, Deserialize, Debug)]
pub struct Description {
    pub value: String,
    pub color: Option<String>,
}

impl Description {
    pub fn is_color(&self, color: &str) -> bool {
        if let Some(description_color) = &self.color {
            description_color.as_str().eq_ignore_ascii_case(color)
        } else {
            false
        }
    }
}

#[derive(DeepSizeOf, Serialize, Deserialize, Debug)]
pub struct Tag {
    pub internal_name: String,
    #[serde(alias = "localized_tag_name")]
    pub name: String,
    pub category: String,
    pub color: Option<String>,
    #[serde(alias = "localized_category_name")]
    pub category_name: Option<String>,
}

#[derive(DeepSizeOf, Serialize, Deserialize, Debug)]
pub struct Action {
    pub name: String,
    pub link: String,
}

#[derive(DeepSizeOf, Serialize, Deserialize, Debug)]
pub struct AppData {
    #[serde(default)]
    #[serde(with = "option_string", rename = "def_index")]
    pub defindex: Option<u32>,
    #[serde(default)]
    #[serde(with = "option_string")]
    pub quantity: Option<u32>,
    #[serde(default)]
    #[serde(with = "option_string")]
    pub quality: Option<u8>,
}

#[derive(DeepSizeOf, Serialize, Deserialize, Debug)]
pub struct ClassInfo {
    #[serde(with = "string")]
    pub classid: u64,
    #[serde(with = "string")]
    pub instanceid: u64,
    pub name: Option<String>,
    pub market_name: String,
    pub market_hash_name: String,
    pub name_color: Option<String>,
    pub background_color: Option<String>,
    pub icon_url: String,
    pub icon_url_large: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(deserialize_with = "into_bool")]
    pub tradable: bool,
    #[serde(deserialize_with = "into_bool")]
    pub marketable: bool,
    #[serde(deserialize_with = "into_bool")]
    pub commodity: bool,
    #[serde(deserialize_with = "string_or_number")]
    pub market_tradable_restriction: u8,
    #[serde(deserialize_with = "string_or_number")]
    pub market_marketable_restriction: u8,
    #[serde(default)]
    #[serde(deserialize_with = "from_fraudwarnings")]
    pub fraudwarnings: Option<Vec<String>>,
    #[serde(default)]
    #[serde(deserialize_with = "hashmap_or_vec")]
    pub descriptions: Vec<Description>,
    #[serde(default)]
    #[serde(deserialize_with = "hashmap_or_vec")]
    pub tags: Vec<Tag>,
    #[serde(default)]
    #[serde(deserialize_with = "hashmap_or_vec")]
    pub actions: Vec<Action>,
    pub app_data: Option<AppData>,
}

pub type ClassInfoMap = HashMap<(u32, u64, u64), Arc<ClassInfo>>;