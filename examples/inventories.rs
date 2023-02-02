use steam_tradeoffer_manager::TradeOfferManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    let manager = TradeOfferManager::builder(
        api_key,
        data_directory,
    )
        .identity_secret(String::from("secret"))
        .build();
    // This method returns only tradable items.
    let inventory = manager.get_my_inventory(440, 2).await?;
    
    println!("{} items in inventory", inventory.len());
    
    if let Some(item) = inventory.first() {
        println!("First item: {}", item.classinfo.market_name);
    }

    Ok(())
}