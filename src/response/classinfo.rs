use serde::Deserialize;
use super::deserializers::{
    from_int_to_bool
};
use crate::serializers::string;

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub category: String,
    pub internal_name: String,
    pub localized_tag_name: String,
    pub localized_category_name: String,
    pub color: Option<String>,
}

#[derive(Deserialize, Debug)]
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
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(deserialize_with = "from_int_to_bool")]
    pub tradable: bool,
    #[serde(deserialize_with = "from_int_to_bool")]
    pub marketable: bool,
    #[serde(deserialize_with = "from_int_to_bool")]
    pub commodity: bool,
    #[serde(deserialize_with = "from_int_to_bool")]
    pub market_tradable_restriction: bool,
    #[serde(deserialize_with = "from_int_to_bool")]
    pub market_marketable_restriction: bool,
    #[serde(default)]
    pub fraudwarnings: Option<Vec<String>>,
    #[serde(default)]
    pub descriptions: Vec<Description>,
    #[serde(default)]
    pub tags: Vec<Tag>,
}
