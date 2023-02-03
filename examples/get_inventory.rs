use steam_tradeoffer_manager::{SteamID, request::GetInventoryOptions, get_inventory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let steamid = get_steamid("STEAMID_OTHER");
    let options = GetInventoryOptions::builder(
        steamid,
        440,
        2,
    ).build();
    // Getting a user's inventory can be done using the manager but it is also provided as a 
    // stand-alone method.
    let inventory = get_inventory(&options).await?;
    
    println!("{} items in inventory", inventory.len());
    
    if let Some(item) = inventory.first() {
        println!("First item: {}", item.classinfo.market_name);
    }

    Ok(())
}

fn get_steamid(key: &str) -> SteamID {
    dotenv::dotenv().ok();
    
    let sid_str = std::env::var(key)
        .unwrap_or_else(|_| panic!("{key} missing"));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}
