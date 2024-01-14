pub(crate) mod helpers;

use crate::response::ClassInfo;
use crate::types::ClassInfoClass;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lfu_cache::LfuCache;

type LfuClassInfoMap = LfuCache<ClassInfoClass, Arc<ClassInfo>>;

/// Used for storing caches of [`ClassInfo`] data in memory. Data is stored using an [`LfuCache`]
/// to limit how many elements are stored in memory. While you probably won't need to use this
/// directly, it is used internally by [`TradeOfferManager`][crate::TradeOfferManager] for 
/// managing [`ClassInfo`] data.
/// 
/// Internally the cache is stored in an [`Arc`] wrapped in a [`Mutex`]. This allows you to clone 
/// the cache and share it between multiple instances of 
/// [`TradeOfferManager`][crate::TradeOfferManager].
/// 
/// # Examples
/// ```
/// use steam_tradeoffer_manager::{TradeOfferManager, ClassInfoCache};
/// 
/// let classinfo_cache = ClassInfoCache::with_capacity(5000);
/// let builder = TradeOfferManager::builder("API_KEY".into(), "./assets")
///    .classinfo_cache(classinfo_cache.clone());
/// // While you could just clone the builder, this demonstrates the utility of re-using the same 
/// // cache.
/// let another_builder = TradeOfferManager::builder("API_KEY".into(), "./assets")
///    .classinfo_cache(classinfo_cache.clone());
/// ````
#[derive(Debug, Clone)]
pub struct ClassInfoCache {
    inner: Arc<Mutex<LfuClassInfoMap>>,
}

impl Default for ClassInfoCache {
    fn default() -> Self {
        Self::with_capacity(2000)
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
        let mut map = HashMap::new();
        // Iterate over the classes and insert them into the map if they exist in the cache.
        // Collect the classes that were not found in the cache.
        let misses = classes
            .into_iter()
            .filter(|class| if let Some(classinfo) = inner.get(class).map(Arc::clone) {
                map.insert(**class, classinfo);
                false
            } else {
                true
            })
            .collect();
        
        (map, misses)
    }
    
    /// Inserts a [`HashMap`] of [`ClassInfo`] data into the cache.
    pub fn insert_map(
        &self,
        classinfos: &HashMap<ClassInfoClass, Arc<ClassInfo>>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        
        for (class, classinfo) in classinfos {
            inner.insert(*class, Arc::clone(classinfo));
        }
    }
}