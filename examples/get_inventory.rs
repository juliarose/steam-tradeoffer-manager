use steam_tradeoffer_manager::{SteamID, request::GetInventoryOptions, get_inventory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let steamid = SteamID::from(u64::from(
        std::env::var("STEAMID_OTHER").unwrap().parse::<u64>().unwrap()
    ));
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