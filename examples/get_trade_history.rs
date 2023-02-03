use steam_tradeoffer_manager::{
    TradeOfferManager,
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
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    // A data directory is required for maintaining state.
    let manager = TradeOfferManager::new(api_key, "../assets");
    let mut options = GetTradeHistoryOptions::default();
    
    options.max_trades = 1;
    
    // Gets your last trade.
    let trades = manager.get_trade_history(&options).await?.trades;
    let trade = trades.into_iter().next().unwrap();
    
    println!("Trade #{}", trade.tradeid);
    println!("Received: {:?}", assets_item_names(&trade.assets_received));
    println!("Given: {:?}", assets_item_names(&trade.assets_given));
    
    Ok(())
}