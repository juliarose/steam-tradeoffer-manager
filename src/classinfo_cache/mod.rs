pub(crate) mod helpers;

use crate::response::ClassInfo;
use crate::types::ClassInfoClass;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lfu_cache::LfuCache;

type LfuClassInfoMap = LfuCache<ClassInfoClass, Arc<ClassInfo>>;

const DEFAULT_CACHE_SIZE: usize = 1000;

/// Used for storing caches of [`ClassInfo`] data in memory. Data is stored using an [`LfuCache`]
/// to limit how many elements are stored in memory. While you probably won't need to use this
/// directly, it is used internally by [`TradeOfferManager`][crate::TradeOfferManager] for 
/// managing [`ClassInfo`] data.
/// 
/// Internally the cache is wrapped in an `Arc<Mutex<T>>`. This allows you to clone the 
/// [`ClassInfoCache`] and share it between multiple instances of 
/// [`TradeOfferManager`][crate::TradeOfferManager] to reduce file reads and memory usage.
/// 
/// # Examples
/// ```
/// use steam_tradeoffer_manager::{TradeOfferManager, ClassInfoCache};
/// 
/// let classinfo_cache = ClassInfoCache::with_capacity(5000);
/// let builder = TradeOfferManager::builder()
///    .classinfo_cache(classinfo_cache.clone());
/// ```
#[derive(Debug, Clone)]
pub struct ClassInfoCache {
    inner: Arc<Mutex<LfuClassInfoMap>>,
}

impl Default for ClassInfoCache {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CACHE_SIZE)
    }
}

impl ClassInfoCache {
    /// Creates a new [`ClassInfoCache`] with the given `capacity`.
    pub fn with_capacity(
        capacity: usize,
    ) -> Self {
        let map = LfuClassInfoMap::with_capacity(capacity);
        
        Self {
            inner: Arc::new(Mutex::new(map)),
        }
    }
    
    /// Gets a map of [`ClassInfo`] wrapped in an [`Arc`] from the cache. The second element of 
    /// the returned tuple is a [`Vec`] of classes that were not found in the cache.
    pub fn get_map<'a>(
        &self,
        classes: &'a [ClassInfoClass],
    ) -> (HashMap<ClassInfoClass, Arc<ClassInfo>>, Vec<&'a ClassInfoClass>) {
        let mut inner = self.inner.lock().unwrap();
        
        classes
            .iter()
            .fold((HashMap::new(), Vec::new()), |mut output, class| {
                if let Some(classinfo) = inner.get(class).map(Arc::clone) {
                    // Insert into the map if a classinfo exists in the cache.
                    output.0.insert(*class, classinfo);
                } else {
                    // Collect the classes that were not found in the cache.
                    output.1.push(class);
                }
                
                output
            })
    }
    
    /// Inserts a [`HashMap`] of [`ClassInfo`] data into the cache.
    pub fn insert_map(
        &self,
        classinfos: HashMap<ClassInfoClass, Arc<ClassInfo>>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        
        for (class, classinfo) in classinfos {
            inner.insert(class, classinfo);
        }
    }
}