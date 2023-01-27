mod types;
pub mod helpers;

use crate::{response::ClassInfo, types::ClassInfoClass};
use std::{sync::Arc, collections::HashMap};
use lfu_cache::LfuCache;

type LfuClassInfoMap = LfuCache<ClassInfoClass, Arc<ClassInfo>>;

/// Used for storing caches for [`ClassInfo`] data. Data is stored using an [`LfuCache`]
/// to limit how many elements are stored in memory.
#[derive(Debug)]
pub struct ClassInfoCache {
    map: LfuClassInfoMap,
}

impl Default for ClassInfoCache {
    fn default() -> Self {
        Self {
            map: create_map(500),
        }
    }
}

impl ClassInfoCache {
    /// Crease a new [`ClassInfoCache`] with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            map: create_map(capacity),
        }
    }
    
    /// Gets a [`ClassInfo`] wrapped in an [`Arc`] from the cache.
    pub fn get(
        &mut self,
        class: &ClassInfoClass,
    ) -> Option<Arc<ClassInfo>> {
        self.map.get(class).map(Arc::clone)
    }
    
    /// Inserts a [`ClassInfo`] into the cache.
    pub fn insert(
        &mut self,
        class: ClassInfoClass,
        classinfo: Arc<ClassInfo>,
    ) {
        self.map.insert(class, classinfo);
    }
    
    /// Inserts a [`HashMap`] of [`ClassInfo`] data into the cache.
    pub fn insert_classinfos(
        &mut self,
        classinfos: &HashMap<ClassInfoClass, Arc<ClassInfo>>,
    ) {
        for (class, classinfo) in classinfos {
            self.map.insert(*class, Arc::clone(classinfo));
        }
    }
}

fn create_map(capacity: usize) -> LfuClassInfoMap {
    LfuCache::with_capacity(capacity)
}