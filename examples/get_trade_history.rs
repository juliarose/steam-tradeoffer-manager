use steam_tradeoffer_manager::{
    TradeOfferManager,
    response::TradeAsset,
    request::GetTradeHistoryOptions,
};

fn assets_item_names(assets: &Vec<TradeAsset>) -> Vec<&str> {
    assets.iter().map(|item| item.classinfo.market_name.as_ref()).collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    // A data directory is required for maintaining state.
    let manager = TradeOfferManager::new(api_key, "./assets");
    let options = GetTradeHistoryOptions {
        max_trades: 1,
        ..GetTradeHistoryOptions::default()
    };
    // Gets your last trade.
    let trades = manager.get_trade_history(&options).await?.trades;
    let trade = trades.into_iter().next().unwrap();
    
    println!("Trade #{}", trade.tradeid);
    println!("Received: {:?}", assets_item_names(&trade.assets_received));
    println!("Given: {:?}", assets_item_names(&trade.assets_given));
    
    Ok(())
}