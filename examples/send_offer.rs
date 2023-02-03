use steam_tradeoffer_manager::{TradeOfferManager, request::NewTradeOffer, SteamID};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (api_key, cookies) = get_session();
    let steamid_other = get_steamid("STEAMID_OTHER");
    let manager = TradeOfferManager::new(api_key, "../assets");
    // This method returns only tradable items.
    let inventory = manager.get_inventory(&steamid_other, 440, 2).await?;
    let items = inventory.into_iter().take(5);
    let offer = NewTradeOffer::builder(steamid_other)
        // Any items that implement Into<NewTradeOfferItem> are fine.
        .items_to_receive(items)
        .build();
        
    manager.set_cookies(&cookies);
    manager.send_offer(&offer).await?;
    
    Ok(())
}

fn get_steamid(key: &str) -> SteamID {
    let sid_str = std::env::var(key)
        .unwrap_or_else(|_| panic!("{key} missing"));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}

/// Gets session from environment variable.
fn get_session() -> (String, Vec<String>) {
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    let cookies = std::env::var("COOKIES").expect("COOKIES missing")
        .split('&')
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    (api_key, cookies)
}