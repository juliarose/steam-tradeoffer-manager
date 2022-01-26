#[macro_use]
extern crate dotenv_codegen;

use dotenv::dotenv;
use steam_tradeoffers::{
    Item,
    SteamTradeOfferAPI,
    response as offers_response,
    request as offers_request
};
use std::{
    fs::File,
    io::Read,
    thread,
    time,
    collections::HashMap,
};
use steamid_ng::SteamID;
use deepsize::DeepSizeOf;

fn is_key(classinfo: &offers_response::ClassInfo) -> bool {
    classinfo.market_hash_name == "Mann Co. Supply Crate Key"
}

fn get_cookies(hostname: &str, filepath: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut file = File::open(filepath).unwrap();
    let mut data = String::new();
    
    file.read_to_string(&mut data).unwrap();
    
    let json: HashMap<String, Vec<String>> = serde_json::from_str(&data).expect("JSON was not well-formatted");
    let values = json.get(hostname).expect("No cookies for hostname");
    
    Ok(values.to_owned())
}

#[allow(unused_variables)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let steam_api_key: String = dotenv!("STEAM_API_KEY").to_string();
    let cookies_json_path: String = dotenv!("COOKIES_JSON_PATH").to_string();
    let mut api = SteamTradeOfferAPI::new(steam_api_key);
    let hostname = "steamcommunity.com";
    let cookies = get_cookies(hostname, &cookies_json_path)?;
    let url = format!("https://{}", hostname);
    
    api.set_cookies(&cookies);
    
    let steamid = SteamID::from(76561198080179568);
    
    // match api.send_offer(&offers_request::CreateTradeOffer {
    //     id: None,
    //     items_to_receive: Vec::new(),
    //     items_to_give: vec![
    //         Item {
    //             appid: 440,
    //             contextid: 2,
    //             amount: 1,
    //             assetid: 10863796759,
    //         }
    //     ],
    //     message: Some("hello from rust".to_string()),
    //     partner: steamid,
    //     token: None,
    // }).await {
    //     Ok(res) => {
    //         println!("{:?}", res);
    //     },
    //     Err(err) => println!("{}", err),
    // }
    // thread::sleep(time::Duration::from_secs(10));
    
    // match api.get_inventory_old(&steamid, 440, 2, true).await {
    //     Ok(items) => {
    //         // println!("{}", items.capacity() * std::mem::size_of::<offers_response::Asset>());
    //         // println!("{}", std::mem::size_of::<offers_response::ClassInfo>());
    //         println!("{:?}", items);
    //         if let Some(item) = items.iter().find(|item| is_key(&*item.classinfo)) {
    //             // match api.send_offer(&offers_request::CreateTradeOffer {
    //             //     id: None,
    //             //     items_to_receive: vec![
    //             //         Item {
    //             //             appid: 440,
    //             //             contextid: 2,
    //             //             amount: 1,
    //             //             assetid: item.assetid,
    //             //         }
    //             //     ],
    //             //     items_to_give: Vec::new(),
    //             //     message: Some("give me that key".to_string()),
    //             //     partner: steamid,
    //             //     token: None,
    //             // }).await {
    //             //     Ok(res) => println!("{:?}", res),
    //             //     Err(err) => println!("{}", err),
    //             // }
    //         } else {
    //             println!("Can't find that :(");
    //         }
    //     },
    //     Err(err) => println!("{}", err),
    // }
    
    match api.get_trade_offers().await {
        Ok(offers) => {
            println!("{:?}", offers);
        },
        Err(err) => println!("{}", err),
    }
    
    // match api.get_asset_classinfos(&vec![(440, 101785959, 11040578)]).await {
    //     Ok(response) => {
    //         println!("{:?}", response);
    //     },
    //     Err(err) => println!("{}", err),
    // }

    // thread::sleep(time::Duration::from_secs(10));
        
    Ok(())
}
