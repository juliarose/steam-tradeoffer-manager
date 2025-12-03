use steam_tradeoffer_manager::SteamID;
use steam_tradeoffer_manager::request::GetInventoryOptions;
use steam_tradeoffer_manager::get_inventory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let steamid: SteamID = std::env::var("STEAMID_OTHER")?
        .parse::<u64>()
        .unwrap()
        .try_into()?;
    let options = GetInventoryOptions::new(steamid, 440, 2);
    // Getting a user's inventory can be done using the manager but it is also provided as a
    // stand-alone method.
    let inventory = get_inventory(&options).await?;
    
    println!("{} items in inventory", inventory.len());
    
    if let Some(item) = inventory.first() {
        println!("First item: {}", item.classinfo.market_name);
    }
    
    Ok(())
}
