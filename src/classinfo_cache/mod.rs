mod types;
pub mod helpers;

use crate::{
    response::classinfo::ClassInfo,
    types::{
        ClassInfoMap,
        ClassInfoAppClass,
        ClassInfoClass,
        AppId,
    }
};
use std::{sync::Arc, collections::HashMap};
use lfu_cache::LfuCache;

/// Used for storing caches for [`ClassInfo`] data. Data is stored using an [`LfuCache`]
/// to limit how many elements are stored in memory.
#[derive(Debug)]
pub struct ClassInfoCache {
    map: LfuCache<ClassInfoClass, Arc<ClassInfo>>,
}

impl Default for ClassInfoCache {
    
    fn default() -> Self {
        let map: LfuCache<ClassInfoClass, Arc<ClassInfo>> = LfuCache::with_capacity(500);
        
        Self {
            map
        }
    }
}

impl ClassInfoCache {
    
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Gets a [`ClassInfo`] wrapped in an [`Arc`] from the cache.
    pub fn get_classinfo(
        &mut self,
        class: &ClassInfoClass,
    ) -> Option<Arc<ClassInfo>> {
        self.map.get(class).map(Arc::clone)
    }
    
    /// Inserts a [`ClassInfo`] into the cache.
    pub fn insert(
        &mut self,
        class: ClassInfoClass,
        classinfo: ClassInfo,
    ) {
        self.map.insert(class, Arc::new(classinfo));
    }
    
    /// Inserts a set of [`ClassInfo`] elements into the cache from JSON strings. This deserializes
    /// the JSON and store a copy of the JSON string to file for reading on-demand.
    pub fn insert_classinfos(
        &mut self,
        appid: AppId,
        classinfos: &HashMap<ClassInfoAppClass, String>,
    ) -> Result<ClassInfoMap, serde_json::Error> {
        let mut map = HashMap::new();
        
        for ((classid, instanceid), classinfo_string) in classinfos {
            let classinfo = serde_json::from_str(classinfo_string)?;
            let classinfo = Arc::new(classinfo);
            let class = (appid, *classid, *instanceid);
            
            self.map.insert(class, Arc::clone(&classinfo));
            map.insert(class, Arc::clone(&classinfo));
        }

        Ok(map)
    }
}