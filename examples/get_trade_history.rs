use steam_tradeoffer_manager::TradeOfferManager;
use steam_tradeoffer_manager::response::TradeAsset;
use steam_tradeoffer_manager::request::GetTradeHistoryOptions;

fn assets_item_names(assets: &[TradeAsset]) -> String {
    assets
        .iter()
        .map(|item| item.classinfo.market_name.as_str())
        .collect::<Vec<_>>()
        .join("\n ")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("API_KEY")?;
    let manager = TradeOfferManager::builder()
        .api_key(api_key)
        .build();
    // Gets your last trade.
    let trades = manager.get_trade_history(&GetTradeHistoryOptions {
        max_trades: 1,
        ..GetTradeHistoryOptions::default()
    }).await?.trades;
    let trade = trades.into_iter().next().unwrap();
    
    println!("Trade #{}", trade.tradeid);
    println!("Received: {:?}", assets_item_names(&trade.assets_received));
    println!("Given: {:?}", assets_item_names(&trade.assets_given));
    
    Ok(())
}