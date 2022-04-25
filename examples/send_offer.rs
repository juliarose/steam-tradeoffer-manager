use steam_tradeoffers::{
    TradeOfferManager,
    request::trade_offer::{NewTradeOffer, Item},
    SteamID,
};
use dotenv::dotenv;
use std::env;

fn get_session() -> (String, Vec<String>) {
    let mut sessionid = None;
    let mut cookies: Vec<String> = Vec::new();
    let cookies_str = env::var("COOKIES")
        .expect("COOKIES missing");
    
    for cookie in cookies_str.split("&") {
        let mut split = cookie.split("=");
        
        if split.next().unwrap() == "sessionid" {
            sessionid = Some(split.next().unwrap().to_string());
        }
        
        cookies.push(cookie.to_string());
    }
    
    (sessionid.unwrap(), cookies)
}

fn get_steamid(key: &str) -> SteamID {
    let sid_str = env::var(key)
        .expect(&format!("{} missing", key));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let steamid = get_steamid("STEAMID");
    let steamid_other = get_steamid("STEAMID_OTHER");
    let key = env::var("API_KEY").expect("API_KEY missing");
    let manager = TradeOfferManager::builder(steamid, key)
        .build();
    let (sessionid, cookies) = get_session();
    let offer = NewTradeOffer::builder(steamid_other)
        .items_to_receive(vec![
            Item {
                appid: 440,
                contextid: 2,
                amount: 1,
                assetid: 11482399896,
            },
        ])
        .build();
        
    manager.set_session(&sessionid, &cookies)?;
    manager.send_offer(&offer).await?;
    
    Ok(())
}