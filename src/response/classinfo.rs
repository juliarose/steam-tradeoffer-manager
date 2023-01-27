use serde::{Serialize, Deserialize};
use crate::deserializers::{
    into_bool,
    hashmap_or_vec,
    from_fraudwarnings,
    string_or_number,
};
use crate::{
    types::{AppId, ClassId, InstanceId},
    serializers::{string, option_string_0_as_none},
};

/// Contains details about an item including names and descriptions.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ClassInfo {
    #[serde(default)]
    /// The item's app ID. some cases this is included
    pub appid: Option<AppId>,
    #[serde(with = "string")]
    /// The ID for this classinfo.
    pub classid: ClassId,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "option_string_0_as_none")]
    /// The specific instance for this classinfo.
    pub instanceid: InstanceId,
    /// The name of the item.
    pub name: String,
    /// The name of the item on the Steam Community Market.
    pub market_name: String,
    /// The market hash name. This is used to link to the on the Steam Community Market.
    pub market_hash_name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The color of the item's name.
    pub name_color: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The background color for the item.
    pub background_color: Option<String>,
    /// The URL to the icon for the item.
    pub icon_url: String,
    /// The URL to the large icon for the item.
    pub icon_url_large: String,
    #[serde(rename = "type")]
    /// The item's type. This is displayed underneath the name of the game in inventories.
    pub r#type: String,
    #[serde(deserialize_with = "into_bool")]
    /// Whether this item can be traded or not.
    pub tradable: bool,
    #[serde(deserialize_with = "into_bool")]
    /// Whether this item is marketable or not.
    pub marketable: bool,
    #[serde(deserialize_with = "into_bool")]
    /// Whether this item is a commodity item on the Steam Community Market.
    pub commodity: bool,
    #[serde(default)]
    #[serde(deserialize_with = "string_or_number")]
    /// How many days the item has left until it can be traded.
    pub market_tradable_restriction: u32,
    #[serde(default)]
    #[serde(deserialize_with = "string_or_number")]
    /// How many days the item has left until it can be listed on the Steam Community Market.
    pub market_marketable_restriction: u32,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "from_fraudwarnings")]
    /// Fraud warnings for this item.
    pub fraudwarnings: Option<Vec<String>>,
    #[serde(default)]
    #[serde(deserialize_with = "hashmap_or_vec")]
    /// Descriptions for this item.
    pub descriptions: Vec<Description>,
    #[serde(default)]
    #[serde(deserialize_with = "hashmap_or_vec")]
    /// Tags for this item.
    pub tags: Vec<Tag>,
    #[serde(default)]
    #[serde(deserialize_with = "hashmap_or_vec")]
    /// Actions for this item.
    pub actions: Vec<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// This contains extra data from the app's internal schema and is sometimes missing depending 
    /// on which endpoint was used.
    pub app_data: AppData,
}

impl ClassInfo {
    /// Convenience method for getting a value out of app_data.
    pub fn get_app_data_value(&self, key: &str) -> Option<&serde_json::Value> {
        if let Some(app_data) = &self.app_data {
            app_data.get(key)
        } else {
            None
        }
    }
    
    /// Convenience method for getting a value out of app_data. Parses string into generic.
    pub fn get_app_data_value_parsed<T>(&self, key: &str) -> Option<T>
    where
        T: std::str::FromStr
    {
        if let Some(app_data) = &self.app_data {
            app_data.get(key).and_then(parse_value)
        } else {
            None
        }
    }
    
    /// Gets def_index value out of app_data parsed as a u64.
    pub fn get_app_data_defindex(&self) -> Option<u64> {
        self.get_app_data_value_parsed("def_index")
    }
    
    /// Gets quality value out of app_data parsed as a u64.
    pub fn get_app_data_quality(&self) -> Option<u64> {
        self.get_app_data_value_parsed("quality")
    }
}

/// A color.
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

/// A tag.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Tag {
    /// The game's internal name of this tag; e.g. for Team Fortress 2 items: "Unique" for items 
    /// under the "Quality" category or "primary" for items under the "Type" category.
    pub internal_name: String,
    #[serde(alias = "localized_tag_name")]
    /// The name of this tag; e.g. for Team Fortress 2 items: "Unique" for items under the 
    /// "Quality" category or "Primary weapon" for items under the "Type" category.
    pub name: String,
    /// The category of this tag; e.g. for Team Fortress the "Quality" category.
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The color associated with this tag.
    pub color: Option<String>,
    #[serde(alias = "localized_category_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The category name of this tag. This is usually the same as category but can sometimes be 
    /// different and is not always present.
    pub category_name: Option<String>,
}

/// An action.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Action {
    /// The name of the action.
    pub name: String,
    /// The link for the example. For example, linking to the item's wiki page for Team Fortress 2 
    /// items.
    pub link: String,
}

/// App data.
pub type AppData = Option<serde_json::Map<String, serde_json::value::Value>>;

fn parse_value<T>(value: &serde_json::Value) -> Option<T> 
where
    T: std::str::FromStr
{
    match value {
        serde_json::Value::String(string) => string.parse::<T>().ok(),
        _ => None,
    }
}

mod tests {
    #[test]
    fn parses_csgo_item() {
        let classinfo: super::ClassInfo = serde_json::from_str(include_str!("fixtures/classinfo_csgo.json")).unwrap();

        assert_eq!(classinfo.tradable, true);
    }
}