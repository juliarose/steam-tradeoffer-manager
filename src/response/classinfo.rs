use serde::{Serialize, Deserialize};
use super::deserializers::{
    into_bool,
    hashmap_or_vec,
    from_fraudwarnings,
    string_or_number,
};
use crate::{
    types::{ClassId, InstanceId},
    serializers::{string, option_string_0_as_none},
};

pub type Color = String;

/// A description.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Description {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// A string representing the color e.g. `"FFFFFF"`
    pub color: Option<Color>,
}

impl Description {
    /// Checks if description color matches string.
    pub fn is_color(&self, color: &str) -> bool {
        if let Some(description_color) = &self.color {
            description_color.eq_ignore_ascii_case(color)
        } else {
            false
        }
    }
    
    /// Checks if description color matches string.
    pub fn is_color_str(&self, color: &str) -> bool {
        self.is_color(color)
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

pub type AppData = Option<serde_json::Map<String, serde_json::value::Value>>;

/// Contains details about an item including names and descriptions. For help 
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ClassInfo {
    #[serde(with = "string")]
    /// The ID for this [`ClassInfo`].
    pub classid: ClassId,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "option_string_0_as_none")]
    /// The specific instance of this [`ClassInfo`].
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
    pub app_data: AppData,
}

fn parse_value_as_u64(value: &serde_json::Value) -> Option<u64> {
    match value {
        serde_json::Value::String(string) => string.parse::<u64>().ok(),
        _ => None,
    }
}

impl ClassInfo {
    /// Convenience  method for getting a value out of app_data.
    pub fn get_app_data_value(&self, key: &str) -> Option<&serde_json::Value> {
        if let Some(app_data) = &self.app_data {
            app_data.get(key)
        } else {
            None
        }
    }
    
    /// Gets def_index value out of app_data parsed as a u64.
    pub fn get_app_data_defindex(&self) -> Option<u64> {
        self.get_app_data_value("def_index")
            .and_then(parse_value_as_u64)
    }
    
    /// Gets quality value out of app_data parsed as a u64.
    pub fn get_app_data_quality(&self) -> Option<u64> {
        self.get_app_data_value("quality")
            .and_then(parse_value_as_u64)
    }
}