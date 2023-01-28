use steam_tradeoffer_manager::{TradeOfferManager, SteamID};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let steamid = get_steamid("STEAMID");
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    let manager = TradeOfferManager::builder(
        steamid,
        api_key,
        data_directory,
    )
        .identity_secret(String::from("secret"))
        .build();
    let inventory = manager.get_inventory(
        &steamid,
        440,
        2,
        true,
    ).await?;
    
    println!("{} items in inventory", inventory.len());
    
    if let Some(item) = inventory.first() {
        println!("First item: {}", item.classinfo.market_name);
    }

    Ok(())
}

fn get_steamid(key: &str) -> SteamID {
    dotenv::dotenv().ok();
    
    let sid_str = std::env::var(key)
        .unwrap_or_else(|_| panic!("{} missing", key));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}