//! Contains custom serialization and deserialization functions.

use crate::response::ClassInfo;
use crate::types::{ClassId, ClassInfoAppClass, ClassInfoAppMap, ClassInfoMap};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::marker::PhantomData;
use std::fmt::{self, Display};
use steamid_ng::SteamID;
use serde::{Serializer, Deserialize};
use serde::de::{self, MapAccess, Visitor, SeqAccess, Deserializer, Unexpected};

pub fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

pub mod ts_seconds_option_none_when_zero {
    use core::fmt;
    use serde::{de, ser};
    use chrono::{DateTime, Utc, serde::SecondsTimestampVisitor};
    
    pub fn serialize<S>(opt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *opt {
            Some(ref dt) => serializer.serialize_some(&dt.timestamp()),
            None => serializer.serialize_none(),
        }
    }
    
    pub fn deserialize<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_option(OptionSecondsTimestampVisitor)
    }
    
    struct OptionSecondsTimestampVisitor;
    
    impl<'de> de::Visitor<'de> for OptionSecondsTimestampVisitor {
        type Value = Option<DateTime<Utc>>;
        
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp in seconds or none")
        }
        
        /// Deserialize a timestamp in seconds since the epoch
        fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            match d.deserialize_i64(SecondsTimestampVisitor) {
                Ok(date) if date.timestamp() == 0 => Ok(None),
                Ok(date) => Ok(Some(date)),
                Err(error) => Err(error),
            }
        }
        
        /// Deserialize a timestamp in seconds since the epoch
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        /// Deserialize a timestamp in seconds since the epoch
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }
}

pub fn string_or_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + TryFrom<u64> + Deserialize<'de>,
    T::Err: Display,
{
    struct NumericVisitor<T> {
        marker: PhantomData<T>,
    }
    
    impl<T> NumericVisitor<T> {
        pub fn new() -> Self {
            Self {
                marker: PhantomData,
            }
        }
    }
    
    impl<'de, T> de::Visitor<'de> for NumericVisitor<T>
    where
        T: FromStr + TryFrom<u64> + Deserialize<'de>,
        T::Err: Display,
    {
        type Value = T;
    
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer or a string")
        }
    
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match T::try_from(v) {
                Ok(c) => {
                    Ok(c)
                },
                Err(_e) => {
                    Err(de::Error::custom("Number too large to fit in target type"))
                }
            }
        }
    
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse::<T>().map_err(de::Error::custom)
        }
    }
    
    deserializer.deserialize_any(NumericVisitor::new())
}

pub fn from_int_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

pub fn from_fraudwarnings<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FraudWarningsVisitor;

    impl<'de> de::Visitor<'de> for FraudWarningsVisitor {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "" => Ok(None),
                other => Ok(Some(vec![other.to_string()])),
            }
        }
        
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut items = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            
            while let Some(item) = seq.next_element::<String>()? {
                items.push(item);
            }
            
            Ok(Some(items))
        }
        
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut items = Vec::new();
            
            while let Some((_key, v)) = access.next_entry::<String, String>()? {
                items.push(v);
            }
            
            Ok(Some(items))
        }
    }
    
    deserializer.deserialize_any(FraudWarningsVisitor)
}

pub fn into_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct DeserializeBoolVisitor;
    
    impl<'de> de::Visitor<'de> for DeserializeBoolVisitor {
        type Value = bool;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer or a string")
        }   
        
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                other => Err(de::Error::invalid_value(
                    Unexpected::Unsigned(other),
                    &"zero or one",
                )),
            }
        }
        
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "0" => Ok(false),
                "1" => Ok(true),
                other => Err(de::Error::invalid_value(
                    Unexpected::Str(other),
                    &"zero or one",
                )),
            }
        }
        
        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }
    }
    
    deserializer.deserialize_any(DeserializeBoolVisitor)
}

pub fn to_classinfo_map<'de, D>(deserializer: D) -> Result<ClassInfoAppMap, D::Error>
where
    D: Deserializer<'de>,
{
    struct ClassInfoVisitor;
    
    impl<'de> Visitor<'de> for ClassInfoVisitor {
        type Value = ClassInfoAppMap;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of classinfos")
        }
        
        fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: Self::Value = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(classinfo) = seq.next_element::<ClassInfo>()? {
                map.insert((classinfo.classid, classinfo.instanceid), Arc::new(classinfo));
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ClassInfoVisitor)
}

pub fn to_trade_offers_classinfo_map<'de, D>(deserializer: D) -> Result<Option<ClassInfoMap>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ClassInfoVisitor;
    
    impl<'de> Visitor<'de> for ClassInfoVisitor {
        type Value = Option<ClassInfoMap>;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of classinfos")
        }
        
        fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: ClassInfoMap = HashMap::with_capacity(seq.size_hint().unwrap_or(0));
            
            while let Some(classinfo) = seq.next_element::<ClassInfo>()? {
                if let Some(appid) = classinfo.appid {
                    map.insert((appid, classinfo.classid, classinfo.instanceid), Arc::new(classinfo));
                }
            }
            
            Ok(Some(map))
        }
        
        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        
        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }
    
    deserializer.deserialize_seq(ClassInfoVisitor)
}

pub fn hashmap_or_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct HashMapVisitor<T> {
        marker: PhantomData<Vec<T>>,
    }
    
    impl<T> HashMapVisitor<T> {
        pub fn new() -> Self {
            Self {
                marker: PhantomData,
            }
        }
    }
    
    impl<'de, T> Visitor<'de> for HashMapVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;
        
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }
        
        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
    
            while let Some(v) = visitor.next_element::<T>()? {
                vec.push(v);
            }
    
            Ok(vec)
        }
        
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }
        
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "" => Ok(Vec::new()),
                other => Err(de::Error::invalid_value(
                    Unexpected::Str(other),
                    &"zero or one",
                )),
            }
        }
        
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut items = Self::Value::new();
            
            while let Some((_key, v)) = access.next_entry::<String, T>()? {
                items.push(v);
            }
            
            Ok(items)
        }
    }
    
    deserializer.deserialize_any(HashMapVisitor::new())
}

pub fn deserialize_classinfo_map<'de, D>(deserializer: D) -> Result<ClassInfoAppMap, D::Error>
where
    D: Deserializer<'de>,
{
    struct ClassInfoMapVisitor;
    
    impl<'de> Visitor<'de> for ClassInfoMapVisitor {
        type Value = ClassInfoAppMap;
    
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }
        
        fn visit_seq<M>(self, mut _seq: M) -> Result<Self::Value, M::Error>
        where
            M: SeqAccess<'de>,
        {
            Ok(Self::Value::new())
        }
        
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::new();
            
            while let Some(key) = access.next_key::<String>()? {
                // generally the key is a string similar to "101785959_11040578"
                // we want to verify that the key appears to be a classid or classid, instanceid
                let is_digits = key
                    .split('_')
                    .all(|s| s.parse::<ClassId>().is_ok());
                
                if is_digits {
                    let classinfo = access.next_value::<ClassInfo>()?;
                    
                    map.insert((classinfo.classid, classinfo.instanceid), Arc::new(classinfo));
                } else if let Ok(_invalid) = access.next_value::<bool>() {
                    // invalid key - discard
                }
            }
            
            Ok(map)
        }
    }
    
    deserializer.deserialize_any(ClassInfoMapVisitor)
}

pub fn deserialize_classinfo_map_raw<'de, D, T>(deserializer: D) -> Result<Vec<(ClassInfoAppClass, T)>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ClassInfoMapVisitor<T> {
        marker: PhantomData<T>,
    }
    
    impl<T> ClassInfoMapVisitor<T> {
        pub fn new() -> Self {
            Self {
                marker: PhantomData,
            }
        }
    }
    
    impl<'de, T> Visitor<'de> for ClassInfoMapVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<(ClassInfoAppClass, T)>;
    
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }
    
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = Self::Value::new();
            
            while let Some(key) = access.next_key::<String>()? {
                let mut iter = key.split('_');
                
                if let Some(classid_str) = iter.next() {
                    if let Ok(classid) = classid_str.parse::<u64>() {
                        let instanceid = if let Some(instanceid_str) = iter.next() {
                            instanceid_str.parse::<u64>().ok()
                        } else {
                            None
                        };
                        let raw_value = access.next_value::<T>()?;
                        let class = (classid, instanceid);
                        
                        map.push((class, raw_value));
                    } else if let Ok(_invalid) = access.next_value::<()>() {
                        // ignore invalid keys e.g. "success"
                    }
                }
            }
            
            Ok(map)
        }
    }
    
    deserializer.deserialize_any(ClassInfoMapVisitor::new())
}

pub fn option_str_to_number<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + TryFrom<u64> + Deserialize<'de>,
    T::Err: Display,
{
    struct OptionVisitor<T> {
        marker: PhantomData<Vec<T>>,
    }
    
    impl<T> OptionVisitor<T> {
        pub fn new() -> Self {
            Self {
                marker: PhantomData,
            }
        }
    }
    
    impl<'de, T> Visitor<'de> for OptionVisitor<T>
    where
        T: FromStr + TryFrom<u64> + Deserialize<'de>,
        T::Err: Display,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number string")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_bool<E>(self, _v: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match T::try_from(v) {
                Ok(c) => {
                    Ok(Some(c))
                },
                Err(_e) => {
                    Err(de::Error::custom("Number too large to fit in target type"))
                }
            }
        }
        
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.parse::<T>().map_err(de::Error::custom)?))
        }
    }

    deserializer.deserialize_any(OptionVisitor::new())
}

pub mod option_string_or_number {
    use std::fmt::Display;
    use std::str::FromStr;
    use serde::{Serializer, Deserializer, Deserialize};
    use serde::de::{self, Visitor};
    use std::marker::PhantomData;
    
    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match value {
            Some(v) => serializer.collect_str(v),
            None => serializer.serialize_none(),
        }
    }
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr + serde::Deserialize<'de>,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        struct OptionStringOrNumberVisitor<T> {
            marker: PhantomData<fn() -> Option<T>>,
        }
        
        impl<'de, T> Visitor<'de> for OptionStringOrNumberVisitor<T>
        where
            T: FromStr + serde::Deserialize<'de>,
            T::Err: Display,
        {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an optional string or number")
            }
            
            fn visit_none<E>(self) -> Result<Self::Value, E> where E: de::Error {
                Ok(None)
            }
            
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                // Try to deserialize as a number first
                let raw: serde_json::Value = Deserialize::deserialize(deserializer)?;

                match raw {
                    serde_json::Value::String(s) => {
                        let parsed = s.parse::<T>().map_err(de::Error::custom)?;
                        Ok(Some(parsed))
                    }
                    serde_json::Value::Number(n) => {
                        // Convert number to string then parse
                        let s = n.to_string();
                        let parsed = s.parse::<T>().map_err(de::Error::custom)?;
                        Ok(Some(parsed))
                    }
                    _ => Err(de::Error::custom("expected string or number")),
                }
            }
        }
        
        deserializer.deserialize_option(OptionStringOrNumberVisitor {
            marker: PhantomData,
        })
    }
}

pub mod string {
    use std::fmt::Display;
    use std::str::FromStr;
    use serde::{de, Serializer, Deserialize, Deserializer};
    
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

pub mod option_string {
    use std::fmt::Display;
    use std::str::FromStr;
    use serde::{Serializer, Deserialize, Deserializer};
    
    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match value {
            Some(string) => serializer.collect_str(string),
            None => serializer.serialize_none()
        }
    }
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::<String>::deserialize(deserializer)?;
        
        if let Some(v) = s {
            return Ok(Some(v.parse::<T>().map_err(serde::de::Error::custom)?))
        }
            
        Ok(None)
    }
}

pub mod option_string_0_as_none {
    use std::fmt::Display;
    use std::str::FromStr;
    use serde::{Serializer, Deserialize, Deserializer};
    
    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match value {
            Some(string) => serializer.collect_str(string),
            None => serializer.serialize_none()
        }
    }
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::<String>::deserialize(deserializer)?;
        
        if let Some(v) = s {
            return Ok(match v.as_str() {
                "0" => None,
                v => Some(v.parse::<T>().map_err(serde::de::Error::custom)?)
            });
        }
            
        Ok(None)
    }
}

pub fn steamid_as_string<S>(steamid: &SteamID, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&u64::from(*steamid).to_string())
}
