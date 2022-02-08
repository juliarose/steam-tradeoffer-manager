use std::{
    fmt::Display,
    str::FromStr
};
use steamid_ng::SteamID;
use serde::{
    Serializer,
    Deserialize,
    Deserializer
};

pub fn option_str_to_number<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    
    if let Some(v) = s {
        return Ok(Some(v.parse::<T>().map_err(serde::de::Error::custom)?))
    }
        
    Ok(None)
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