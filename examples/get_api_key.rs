use steam_tradeoffer_manager::TradeOfferManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cookies = get_cookies();
    // Just pass in a vec containing your login cookies.
    let api_key = TradeOfferManager::get_api_key(&cookies).await?;
    
    println!("Your Steam Web API key is {api_key}");
    Ok(())
}

/// Gets cookies from environment variable.
fn get_cookies() -> Vec<String> {
    dotenv::dotenv().ok();
    std::env::var("COOKIES").expect("COOKIES missing")
        .split('&')
        .map(|s| s.to_string())
        .collect()
}