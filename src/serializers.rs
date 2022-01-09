
use serde::Serializer;
use steamid_ng::SteamID;

pub mod string {
    use std::fmt::Display;
    use std::str::FromStr;
    use serde::{de, Serializer, Deserialize, Deserializer};
    
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where T: Display,
              S: Serializer
    {
        serializer.collect_str(value)
    }
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where T: FromStr,
              T::Err: Display,
              D: Deserializer<'de>
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

pub fn steamid_as_string<S>(steamid: &SteamID, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    s.serialize_str(&u64::from(steamid.clone()).to_string())
}

pub fn as_string<S, T>(value: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    s.serialize_str(&value.to_string())
}

pub fn option_number_to_str<S, T>(value: &Option<T>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    if let Some(ref v) = *value {
        s.serialize_str(&v.to_string())
    } else {
        s.serialize_none()
    }
}
