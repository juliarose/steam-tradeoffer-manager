use crate::serialize;
use crate::types::{AppId, ClassId, InstanceId, ServerTime};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::hash::Hash;

/// Contains details about an item including names and descriptions.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct ClassInfo {
    /// The item's app ID. This is included when including descriptions in the `GetTradeOffers`
    /// and `GetTradeHistory` response.
    #[serde(default)]
    #[serde(with = "serialize::option_string_or_number")]
    pub appid: Option<AppId>,
    /// The ID for this classinfo.
    #[serde(with = "serialize::string")]
    pub classid: ClassId,
    /// The specific instance ID for this classinfo.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serialize::option_string_0_as_none")]
    pub instanceid: InstanceId,
    /// The name of the item.
    pub name: String,
    /// The name of the item on the Steam Community Market.
    pub market_name: String,
    /// The market hash name. This is used to link to the item on the Steam Community Market. This
    /// is an empty string in some cases, like Steam coupons.
    #[serde(default)]
    pub market_hash_name: Option<String>,
    /// The color of the item's name.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_color: Option<String>,
    /// The background color for the item.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    /// The URL to the icon for the item.
    pub icon_url: String,
    /// The URL to the large icon for the item. This is almost always present but is missing in
    /// rare events.
    pub icon_url_large: Option<String>,
    /// The item's type. This is displayed underneath the name of the game in inventories.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Whether this item can be traded or not.
    #[serde(deserialize_with = "serialize::into_bool")]
    pub tradable: bool,
    /// Whether this item is marketable or not.
    #[serde(deserialize_with = "serialize::into_bool")]
    pub marketable: bool,
    /// Whether this item is a commodity item on the Steam Community Market.
    #[serde(deserialize_with = "serialize::into_bool")]
    pub commodity: bool,
    /// How many days the item has left until it can be traded.
    #[serde(default)]
    #[serde(deserialize_with = "serialize::string_or_number")]
    pub market_tradable_restriction: u32,
    /// How many days the item has left until it can be listed on the Steam Community Market.
    #[serde(default)]
    #[serde(deserialize_with = "serialize::string_or_number")]
    pub market_marketable_restriction: u32,
    /// Fraud warnings for this item.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "serialize::from_fraudwarnings")]
    pub fraudwarnings: Option<Vec<String>>,
    /// Descriptions for this item.
    #[serde(default)]
    #[serde(deserialize_with = "serialize::hashmap_or_vec")]
    pub descriptions: Vec<Description>,
    /// Descriptions only visible to the owner for this item. This can only be obtained when an
    /// access token is provided.
    #[serde(default)]
    #[serde(deserialize_with = "serialize::hashmap_or_vec")]
    pub owner_descriptions: Vec<Description>,
    /// Tags for this item.
    #[serde(default)]
    #[serde(deserialize_with = "serialize::hashmap_or_vec")]
    pub tags: Vec<Tag>,
    /// Actions for this item.
    #[serde(default)]
    #[serde(deserialize_with = "serialize::hashmap_or_vec")]
    pub actions: Vec<Action>,
    /// This contains extra data from the app's internal schema. This is only included in  
    /// `GetAssetClassInfo` and `inventory/json` responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: AppData,
    /// Cache expiration.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serialize::option_string")]
    pub cache_expiration: Option<ServerTime>,
    /// Item expiration.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serialize::option_string")]
    pub item_expiration: Option<ServerTime>,
    /// Whether this item is trade sealed or not. This can only be obtained when an access token is
    /// provided. `false` if not present.
    #[serde(default)]
    #[serde(deserialize_with = "serialize::into_bool")]
    pub sealed: bool,
}

impl ClassInfo {
    /// Convenience method for getting a value from `app_data`.
    pub fn get_app_data_value<Q>(&self, key: &Q) -> Option<&serde_json::Value>
    where
        String: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
    {
        if let Some(app_data) = &self.app_data {
            app_data.get(key)
        } else {
            None
        }
    }
    
    /// Convenience method for parsing a string value from `app_data` into the desired type.
    /// 
    /// # Examples
    /// ```
    /// use steam_tradeoffer_manager::response::ClassInfo;
    /// use serde_json::json;
    /// 
    /// let app_data = serde_json::from_str(r#"{"def_index": "123"}"#).unwrap();
    /// let classinfo = ClassInfo {
    ///     app_data: Some(app_data),
    ///     ..Default::default()
    /// };
    /// 
    /// // Parse def_index as u64
    /// let def_index: Option<u64> = classinfo.get_app_data_value_parsed("def_index");
    /// assert_eq!(def_index, Some(123));
    /// ```
    pub fn get_app_data_value_parsed<Q, T>(&self, key: &Q) -> Option<T>
    where
        String: Borrow<Q>,
        Q: ?Sized + Ord + Eq + Hash,
        T: std::str::FromStr,
    {
        self.get_app_data_value(key).and_then(|value| {
            match value {
                serde_json::Value::String(string) => string.parse::<T>().ok(),
                _ => None,
            }
        })
    }
    
    /// Gets `def_index` value out of app_data parsed as a [`u64`].
    pub fn get_app_data_defindex(&self) -> Option<u64> {
        self.get_app_data_value_parsed("def_index")
    }
    
    /// Gets `quality` value out of app_data parsed as a [`u64`].
    pub fn get_app_data_quality(&self) -> Option<u64> {
        self.get_app_data_value_parsed("quality")
    }
}

/// The type used for colors.
pub type Color = String;

/// Description.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct Description {
    /// Description type. Usually `"text"` or `"html"`. Not always present.
    #[serde(default)]
    pub r#type: Option<String>,
    /// The description message.
    pub value: String,
    /// A string representing the color e.g. `"FFFFFF"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

impl Description {
    /// Checks if description color matches string.
    /// 
    /// # Examples
    /// ```
    /// use steam_tradeoffer_manager::response::Description;
    /// 
    /// let description = Description {
    ///     r#type: Some(String::from("text")),
    ///     value: String::from("Can't be applied with other discounts."),
    ///     color: Some(String::from("ffffff")),
    /// };
    /// 
    /// // case-insensitive
    /// assert!(description.is_color("FFFFFF"));
    /// // prefixed with # is ok too
    /// assert!(description.is_color("#ffffff"));
    /// ```
    pub fn is_color<S: AsRef<str>>(&self, color: S) -> bool {
        if let Some(description_color) = &self.color {
            let color = color.as_ref();
            
            if color.starts_with('#') {
                description_color.eq_ignore_ascii_case(&color[1..color.len()])
            } else {
                description_color.eq_ignore_ascii_case(color)
            }
        } else {
            false
        }
    }
    
    /// Checks if description color matches string.
    pub fn is_color_str<S: AsRef<str>>(&self, color: S) -> bool {
        self.is_color(color)
    }
    
    /// Checks if description color matches another color (represented as a [`u32`]).
    /// 
    /// `false` if the description color can't be parsed as a [`u32`].
    pub fn is_color_int(&self, color: u32) -> bool {
        if let Some(description_color) = &self.color {
            if let Ok(parsed) = u32::from_str_radix(description_color, 16) {
                parsed == color
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Tag.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Tag {
    /// The game's internal name of this tag e.g. for Team Fortress 2 items: "Unique" for items
    /// under the "Quality" category or "primary" for items under the "Type" category.
    pub internal_name: String,
    /// The name of this tag e.g. for Team Fortress 2 items: "Unique" for items under the
    /// "Quality" category or "Primary weapon" for items under the "Type" category. This value
    /// has the alias of `localized_tag_name`.
    #[serde(alias = "localized_tag_name")]
    pub name: String,
    /// The category of this tag e.g. for Team Fortress the "Quality" category.
    pub category: String,
    /// The color associated with this tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// The category name of this tag. This is usually the same as category but can sometimes be
    /// different and is not always present. This value has the alias of `localized_category_name`.
    #[serde(alias = "localized_category_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[cfg(test)]
mod tests {
    #[test]
    fn parses_csgo_item() {
        let classinfo: super::ClassInfo = serde_json::from_str(
            include_str!("fixtures/classinfo_csgo.json"),
        ).unwrap();
        
        assert!(classinfo.tradable);
    }
    
    #[test]
    fn parses_coupon() {
        let classinfo: super::ClassInfo = serde_json::from_str(
            include_str!("fixtures/classinfo_item_expiration.json"),
        ).unwrap();
        
        assert!(classinfo.item_expiration.is_some());
    }
    
    #[test]
    fn is_color_works() {
        let classinfo: super::ClassInfo = serde_json::from_str(
            include_str!("fixtures/classinfo_item_expiration.json"),
        ).unwrap();
        let description = classinfo.descriptions.first().unwrap();
        
        assert!(description.is_color("7a9fc5"));
        assert!(description.is_color("#7a9fc5"));
        assert!(!description.is_color(""));
    }
}
