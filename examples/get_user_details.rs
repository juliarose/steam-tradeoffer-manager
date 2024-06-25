use steam_tradeoffer_manager::{TradeOfferManager, SteamID};
use steam_tradeoffer_manager::enums::GetUserDetailsMethod;
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let steamid: SteamID = std::env::var("STEAMID_OTHER").unwrap().parse::<u64>().unwrap().into();
    let cookies = std::env::var("COOKIES").expect("COOKIES missing")
        .split("; ")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    // An API key isn't needed for this example.
    let manager = TradeOfferManager::builder()
        // Cookies are required for getting user details. These can be included in the builder or 
        // using the `set_cookies` method on the manager.
        .cookies(cookies)
        .build();
    // Passing in GetUserDetailsMethod::None assumes we are friends with the user.
    let user_details = manager.get_user_details(steamid, GetUserDetailsMethod::None).await?;
    
    println!("Trade will result in escrow? {}", user_details.has_escrow().bold());
    Ok(())
}