use hashbrown::HashMap;
use crate::{
    response::asset::Asset,
    types::{AppId, AssetId}
};

type ItemMap = HashMap<(AppId, AssetId), Asset>;

pub struct Inventory {
    values: ItemMap,
    index: usize,
}

// impl<'a> Iterator for Inventory<'a> {
//     type Item = &'a Asset;
    
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.index >= self.values.len() {
//             return None
//         }
        
//         self.index += 1;
//         Some(&self.values[self.index - 1])
//     }
// }