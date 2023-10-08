pub mod helpers;

use crate::response::ClassInfo;
use crate::types::ClassInfoClass;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lfu_cache::LfuCache;

type LfuClassInfoMap = LfuCache<ClassInfoClass, Arc<ClassInfo>>;

/// Used for storing caches of [`ClassInfo`] data. Data is stored using an [`LfuCache`]
/// to limit how many elements are stored in memory.
#[derive(Debug)]
pub struct ClassInfoCache {
    map: LfuClassInfoMap,
}

impl Default for ClassInfoCache {
    fn default() -> Self {
        Self::new(2000)
    }
}

impl ClassInfoCache {
    /// Creates a new [`ClassInfoCache`] with the given `capacity`.
    pub fn new(
        capacity: usize,
    ) -> Self {
        let map = LfuClassInfoMap::with_capacity(capacity);
        
        Self {
            map,
        }
    }
    
    /// Creates a new sharable [`ClassInfoCache`] with the given `capacity`.
    /// 
    /// This simply wraps the cache in an [`Arc`] and [`Mutex`] for sharing.
    pub fn new_shared(
        capacity: usize,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(capacity)))
    }
    
    /// Gets a [`ClassInfo`] wrapped in an [`Arc`] from the cache.
    pub fn get(
        &mut self,
        class: &ClassInfoClass,
    ) -> Option<Arc<ClassInfo>> {
        self.map.get(class).map(Arc::clone)
    }
    
    /// Gets a [`ClassInfo`] from the cache.
    pub fn get_borrowed(
        &mut self,
        class: &ClassInfoClass,
    ) -> Option<&Arc<ClassInfo>> {
        self.map.get(class)
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
    pub fn insert_map(
        &mut self,
        classinfos: &HashMap<ClassInfoClass, Arc<ClassInfo>>,
    ) {
        for (class, classinfo) in classinfos {
            self.map.insert(*class, Arc::clone(classinfo));
        }
    }
}