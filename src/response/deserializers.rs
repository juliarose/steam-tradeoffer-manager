use serde::{
    Deserialize,
    de::{self, Visitor, SeqAccess, Deserializer}
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use super::classinfo::ClassInfo;

pub fn from_int_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u8 = Deserialize::deserialize(deserializer)?;
    
    match s {
        1 => Ok(true),
        _ => Ok(false),
    }
}

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    
    T::from_str(&s).map_err(de::Error::custom)
}

pub fn to_classinfo_map<'de, D>(deserializer: D) -> Result<HashMap<(u64, u64), Arc<ClassInfo>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ClassInfoVisitor;

    impl<'de> Visitor<'de> for ClassInfoVisitor {
        type Value = HashMap<(u64, u64), Arc<ClassInfo>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of classinfos")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<(u64, u64), Arc<ClassInfo>>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<(u64, u64), Arc<ClassInfo>> = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(classinfo) = seq.next_element::<ClassInfo>()? {
                map.insert((classinfo.classid, classinfo.instanceid), Arc::new(classinfo));
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ClassInfoVisitor)
}