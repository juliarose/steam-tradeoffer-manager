use steam_tradeoffer_manager::types::{AppId, ClassId, InstanceId};
use steam_tradeoffer_manager::ClassInfoCache;
use steam_tradeoffer_manager::response::ClassInfo;
use steam_tradeoffer_manager::api::response::RawTradeOffer;
use steam_tradeoffer_manager::error::FileError;
use std::path::{PathBuf, Path};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

type ClassInfoClass = (AppId, ClassId, InstanceId);
type ClassInfoMap = HashMap<ClassInfoClass, Arc<ClassInfo>>;

type ClassInfoFile = (ClassInfoClass, ClassInfo);

fn get_classinfo_file_path(
    class: ClassInfoClass,
    data_directory: &Path, 
) -> Result<PathBuf, FileError> {
    let (appid, classid, instanceid) = class;
    let filename = format!("{}_{}_{}.json", appid, classid, instanceid.unwrap_or(0));
    
    Ok(data_directory.join(filename))
}

fn load_classinfo_sync(
    class: ClassInfoClass,
    data_directory: &Path, 
) -> Result<ClassInfoFile, FileError> {
    let filepath = get_classinfo_file_path(class, data_directory)?;
    let data = std::fs::read_to_string(&filepath)?;
    
    match serde_json::from_str::<ClassInfo>(&data) {
        Ok(classinfo) => {
            Ok((class, classinfo))
        },
        Err(error) => {
            // remove the file...
            let _ = std::fs::remove_file(&filepath);
            
            Err(FileError::Parse(error))
        },
    }
}

fn get_offers() -> Vec<RawTradeOffer> {
    #[derive(Deserialize, Debug)]
    pub struct GetTradeOffersResponseBody {
        #[serde(default)]
        pub trade_offers_sent: Vec<RawTradeOffer>,
        #[serde(default)]
        pub trade_offers_received: Vec<RawTradeOffer>,
        pub next_cursor: Option<u32>,
    }
    
    #[derive(Deserialize, Debug)]
    pub struct GetTradeOffersResponse {
        pub response: GetTradeOffersResponseBody,
    }
    
    let mut response = serde_json::from_str::<GetTradeOffersResponse>(
        include_str!("fixtures/offers.json")
    ).unwrap().response;
    let mut offers = Vec::new();
    
    offers.append(&mut response.trade_offers_received);
    offers.append(&mut response.trade_offers_sent);
    offers
}

fn get_classinfo_cache(
    offers: &[RawTradeOffer],
) -> ClassInfoCache {
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
        .map(|class| {
            let (class, classinfo) = load_classinfo_sync(
                class,
                &classinfos_path,
            ).unwrap();
            
            (class, Arc::new(classinfo))
        })
        .collect::<HashMap<_, _>>();
    let classinfo_cache = ClassInfoCache::with_capacity(500);
    
    classinfo_cache.insert_map(classes);
    classinfo_cache
}

fn get_map(
    classes: &[ClassInfoClass],
    classinfo_cache: &ClassInfoCache,
) -> ClassInfoMap {
    let (map, _misses) = classinfo_cache.get_map(classes);
    map
}

fn criterion_benchmark(c: &mut Criterion) {
    let offers = get_offers();
    let classinfo_cache = get_classinfo_cache(&offers);
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
        .collect::<Vec<_>>();
    
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
            .collect::<Vec<_>>();
        let map = get_map(
            &classes,
            &classinfo_cache,
        );
        let _ = offers
            .clone() // cloning only makes a small detriment in the benchmark (~3%)
            .into_iter()
            .map(|offer| offer.try_combine_classinfos(&map).unwrap())
            .collect::<Vec<_>>();
    }));
    
    c.bench_function("gets classinfo caches from cache", |b| b.iter(|| {
        let _map = get_map(
            &classes,
            &classinfo_cache,
        );
    }));
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark
}

criterion_main!(benches);