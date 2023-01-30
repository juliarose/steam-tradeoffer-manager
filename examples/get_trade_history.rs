use steam_tradeoffer_manager::{
    TradeOfferManager,
    SteamID,
    response::TradeAsset,
    request::GetTradeHistoryOptions,
};

fn assets_item_names<'a>(
    assets: &'a Vec<TradeAsset>,
) -> Vec<&'a str> {
    assets
        .iter()
        .map(|item| item.classinfo.market_hash_name.as_ref())
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let steamid = get_steamid("STEAMID");
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    let manager = TradeOfferManager::builder(
        steamid,
        api_key,
        data_directory,
    ).build();
    let mut options = GetTradeHistoryOptions::default();
    
    options.max_trades = 3;
    
    let trades = manager.get_trade_history(&options).await?.trades;
    let trade = trades.into_iter().next().unwrap();
    
    println!("Trade #{}", trade.tradeid);
    println!("Received: {:?}", assets_item_names(&trade.assets_received));
    println!("Given: {:?}", assets_item_names(&trade.assets_given));
    
    Ok(())
}

fn get_steamid(key: &str) -> SteamID {
    dotenv::dotenv().ok();
    
    let sid_str = std::env::var(key)
        .unwrap_or_else(|_| panic!("{key} missing"));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}