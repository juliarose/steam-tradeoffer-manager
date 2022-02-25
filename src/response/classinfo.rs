use serde::{Serialize, Deserialize};
use super::{
    deserializers::{
        into_bool,
        hashmap_or_vec,
        from_fraudwarnings,
        string_or_number
    }
};
use crate::{
    types::{
        ClassId,
        InstanceId
    },
    serializers::{
        string,
        option_string,
        option_string_0_as_none
    }
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Description {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl Description {
    pub fn is_color(&self, color: &str) -> bool {
        if let Some(description_color) = &self.color {
            description_color.eq_ignore_ascii_case(color)
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Tag {
    pub internal_name: String,
    #[serde(alias = "localized_tag_name")]
    pub name: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(alias = "localized_category_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Action {
    pub name: String,
    pub link: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AppData {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "option_string", rename = "def_index")]
    pub defindex: Option<u32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "option_string")]
    pub quantity: Option<u32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "option_string")]
    pub quality: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ClassInfo {
    #[serde(with = "string")]
    pub classid: ClassId,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "option_string_0_as_none")]
    pub instanceid: InstanceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub market_name: String,
    pub market_hash_name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_color: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<AppData>,
}