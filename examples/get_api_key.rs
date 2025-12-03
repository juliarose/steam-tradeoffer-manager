use steam_tradeoffer_manager::TradeOfferManager;
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let cookies = std::env::var("COOKIES")?
        .split("; ")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    // Just pass in a vec containing your login cookies.
    // ***IMPORTANT***: By calling this method you are agreeing to the Steam Web API Terms of Use: 
    // https://steamcommunity.com/dev/apiterms
    let api_key = TradeOfferManager::get_api_key(&cookies).await?;
    
    println!("Your Steam Web API key is {}", api_key.bold());
    
    Ok(())
}
