use crate::{
    response::asset::Asset,
    types::{AppId, AssetId}
};
use tf2_price::{Currencies, ONE_REF, ONE_REC, ONE_SCRAP};
use std::{
    slice::Iter,
    ops::Deref,
    collections::HashSet
};

type ItemMap = Vec<Asset>;

pub struct ItemCollection(ItemMap);

impl ItemCollection {
    
    pub fn new() -> Self {
        Self(ItemMap::new())
    }
    
    pub fn push(&mut self, asset: Asset) {
        self.0.push(asset);
    }
    
    pub fn into_vec(self) -> Vec<Asset> {
        self.into()
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn filter_app(&self, appid: AppId) -> ItemCollection {
        let assets = self
            .into_iter()
            .filter(|asset| asset.appid == appid)
            .map(|asset| asset.clone())
            .collect::<Vec<_>>();
        
        ItemCollection::from(assets)
    }
    
    pub fn remove_set(&mut self, set: &HashSet<(AppId, AssetId)>) {
        self.0.retain(|asset| !set.contains(&(asset.appid, asset.assetid)));
    }
    
    pub fn append(&mut self, items: &mut ItemCollection) {
        self.0.append(&mut items.0);
    }
    
    pub fn update(&mut self, items: ItemCollection) {
        self.0 = items.0;
    }
}

impl Deref for ItemCollection {
    type Target = Vec<Asset>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> IntoIterator for &'s ItemCollection {
    type Item = &'s Asset;
    type IntoIter = Iter<'s, Asset>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// impl<'s> Iterator for &'s ItemCollection {
    
// }

impl From<ItemCollection> for Vec<Asset> {
    
    fn from(items: ItemCollection) -> Vec<Asset> {
        items.0
    }
}

impl From<Vec<Asset>> for ItemCollection {
    
    fn from(assets: Vec<Asset>) -> ItemCollection {
        ItemCollection(assets)
    }
}

impl From<ItemCollection> for Currencies {
    
    fn from(items: ItemCollection) -> Currencies {
        count_currencies(&items)
    }
}

impl<'a> From<&'a ItemCollection> for Currencies {
    
    fn from(items: &ItemCollection) -> Currencies {
        count_currencies(items)
    }
}

fn count_currencies(items: &ItemCollection) -> Currencies {
    let mut currencies = Currencies::new();
    
    for item in items {
        if item.appid != 440 {
            continue;
        }
        
        match item.classinfo.market_hash_name.as_str() {
            "Mann Co. Supply Crate Key" => currencies.keys += 1,
            "Refined Metal" => currencies.metal += ONE_REF,
            "Reclaimed Metal" => currencies.metal += ONE_REC,
            "Scrap Metal" => currencies.metal += ONE_SCRAP,
            _ => {},
        }
    }
    
    currencies
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::response;
    use tf2_price::refined;
    
    fn create_asset_refined_metal(assetid: u64) -> Asset {
        let classinfo: response::classinfo::ClassInfo = serde_json::from_str(include_str!("response/fixtures/classinfo_refined_metal.json")).unwrap();
        let classinfo = Arc::new(classinfo);
        
        Asset {
            appid: 440,
            contextid: 2,
            assetid,
            amount: 1,
            classinfo: Arc::clone(&classinfo),
        }
    }
    
    #[test]
    fn converts_into_currencies() {
        let items: Vec<_> = (0..10)
            .map(|i| create_asset_refined_metal(i as u64))
            .collect();
        let currencies: Currencies = ItemCollection::from(items).into();
        
        assert_eq!(currencies, Currencies { keys: 0, metal: refined!(10) });
    }
    
    #[test]
    fn iterates_items() {
        let items: Vec<_> = (0..3)
            .map(|i| create_asset_refined_metal(i as u64))
            .collect();
        let assetids = ItemCollection::from(items)
            .into_iter()
            .map(|asset| asset.assetid)
            .collect::<Vec<u64>>();
        
        assert_eq!(assetids, vec![0, 1, 2]);
    }
    
    #[test]
    fn filters_app() {
        let items: Vec<_> = (0..3)
            .map(|i| create_asset_refined_metal(i as u64))
            .collect();
        let items = ItemCollection::from(items).filter_app(440);
        
        assert_eq!(items.len(), 3);
    }
}