use steam_tradeoffer_manager::{
    TradeOfferManager,
    SteamID,
    chrono::Duration,
};
use dotenv::dotenv;
use std::env;

fn get_steamid(key: &str) -> SteamID {
    let sid_str = env::var(key)
        .unwrap_or_else(|_| panic!("{} missing", key));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let steamid = get_steamid("STEAMID");
    let key = env::var("API_KEY").expect("API_KEY missing");
    let manager = TradeOfferManager::builder(steamid, key)
        .identity_secret(String::from("secret"))
        .cancel_duration(Duration::minutes(30))
        .build();
    let inventory = manager.get_inventory(
        &steamid,
        440,
        2,
        true,
    ).await?;
    
    println!("{} items in inventory", inventory.len());
    
    if let Some(item) = inventory.iter().next() {
        println!("First item: {}", item.classinfo.market_name);
    }

    Ok(())
}