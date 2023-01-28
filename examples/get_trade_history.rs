use steam_tradeoffer_manager::{TradeOfferManager, SteamID};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let steamid = get_steamid("STEAMID");
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    let manager = TradeOfferManager::builder(steamid, api_key)
        .identity_secret(String::from("secret"))
        .build();
    let (trades, _more) = manager.get_trade_history(
        1,
        None,
        None,
        false,
        false,
    ).await?;
    
    println!("Last trade: {:?}", trades);
    
    Ok(())
}

fn get_steamid(key: &str) -> SteamID {
    dotenv::dotenv().ok();
    
    let sid_str = std::env::var(key)
        .unwrap_or_else(|_| panic!("{} missing", key));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}