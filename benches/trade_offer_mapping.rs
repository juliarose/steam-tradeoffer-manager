use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;
use steam_tradeoffer_manager::{
    classinfo_cache::{ClassInfoCache, helpers::load_classinfo_sync},
    api::raw, types::{ClassInfoMap, ClassInfoClass},
};
use std::{
    path::PathBuf,
    collections::HashSet,
    sync::{Arc, Mutex},
};

fn get_offers() -> Vec<raw::RawTradeOffer> {
    #[derive(Deserialize, Debug)]
    pub struct GetTradeOffersResponseBody {
        #[serde(default)]
        pub trade_offers_sent: Vec<raw::RawTradeOffer>,
        #[serde(default)]
        pub trade_offers_received: Vec<raw::RawTradeOffer>,
        pub next_cursor: Option<u32>,
    }
    
    #[derive(Deserialize, Debug)]
    pub struct GetTradeOffersResponse {
        pub response: GetTradeOffersResponseBody,
    }
    
    let mut response = serde_json::from_str::<GetTradeOffersResponse>(include_str!("fixtures/offers.json")).unwrap().response;
    let mut offers = Vec::new();
    
    offers.append(&mut response.trade_offers_received);
    offers.append(&mut response.trade_offers_sent);
    offers
}

fn get_classinfo_cache(
    offers: &Vec<raw::RawTradeOffer>,
) -> Arc<Mutex<ClassInfoCache>> {
    let classinfos_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures/classinfos");
    let classes = offers
        .iter()
        .flat_map(|offer| {
            offer.items_to_give
                .iter()
                .chain(offer.items_to_receive.iter())
                .map(|item| (item.appid, item.classid, item.instanceid))
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|class| load_classinfo_sync(
            class,
            &classinfos_path,
        ).unwrap())
        .collect::<Vec<_>>();
    let mut classinfo_cache = ClassInfoCache::new(500);
    
    for (class, classinfo) in classes {
        classinfo_cache.insert(class, Arc::new(classinfo));
    }
    
    Arc::new(Mutex::new(classinfo_cache))
}

fn get_map(
    classes: Vec<ClassInfoClass>,
    classinfo_cache: &Arc<Mutex<ClassInfoCache>>,
) -> ClassInfoMap {
    let mut classinfo_cache = classinfo_cache.lock().unwrap();
    
    classes
        .into_iter()
        .map(|class| {
            let classinfo = classinfo_cache.get(&class).unwrap();
            
            (class, classinfo)
        })
        .collect::<_>()
}

fn criterion_benchmark(c: &mut Criterion) {
    let offers = get_offers();
    let classinfo_cache = get_classinfo_cache(&offers);
    
    c.bench_function("maps items in offer with descriptions", |b| b.iter(|| {
        let classes = offers
            .iter()
            .flat_map(|offer| {
                offer.items_to_give
                    .iter()
                    .chain(offer.items_to_receive.iter())
                    .map(|item| (item.appid, item.classid, item.instanceid))
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let map = get_map(
            classes,
            &classinfo_cache,
        );
        let _ = offers
            .clone()
            .into_iter()
            .map(|offer| offer.try_combine_classinfos(&map).unwrap())
            .collect::<Vec<_>>();
    }));
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark
}

criterion_main!(benches);