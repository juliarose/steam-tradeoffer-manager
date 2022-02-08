use crate::{
    response::ClassInfo,
    types::{
        ClassInfoMap,
        ClassInfoAppClass,
        ClassInfoClass
    }
};
use std::{
    sync::Arc,
    collections::HashMap
};
use lfu_cache::LfuCache;

#[derive(Debug)]
pub struct ClassInfoCache {
    map: LfuCache<ClassInfoClass, Arc<ClassInfo>>,
}

impl ClassInfoCache {
    pub fn new() -> Self {
        let map: LfuCache<ClassInfoClass, Arc<ClassInfo>> = LfuCache::with_capacity(1000);
        
        Self {
            map
        }
    }
    
    pub fn get_classinfo(&mut self, class: &ClassInfoClass) -> Option<Arc<ClassInfo>> {
        match self.map.get(&class) {
            Some(classinfo) => Some(Arc::clone(classinfo)),
            None => None,
        }
    }
    
    // takes the result from `load_classinfos` above and adds it to the map
    pub fn insert(&mut self, class: ClassInfoClass, classinfo: ClassInfo) {
        self.map.insert(class, Arc::new(classinfo));
    }
    
    pub fn insert_classinfos(&mut self, appid: u32, classinfos: &HashMap<ClassInfoAppClass, String>) -> Result<ClassInfoMap, serde_json::Error> {
        let mut map = HashMap::new();
        
        for ((classid, instanceid), classinfo_string) in classinfos {
            println!("{}", classinfo_string);
            let classinfo = serde_json::from_str(classinfo_string)?;
            let classinfo = Arc::new(classinfo);
            let class = (appid.clone(), classid.clone(), instanceid.clone());
            
            self.map.insert(class, Arc::clone(&classinfo));
            map.insert(class, Arc::clone(&classinfo));
        }

        Ok(map)
    }
}