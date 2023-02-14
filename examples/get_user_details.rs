use steam_tradeoffer_manager::{TradeOfferManager, SteamID, enums::GetUserDetailsMethod};
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let steamid: SteamID = std::env::var("STEAMID_OTHER").unwrap().parse::<u64>().unwrap().into();
    let cookies = std::env::var("COOKIES").expect("COOKIES missing")
        .split("; ")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    // A blank key can be passed, it isn't needed in this example.
    let manager = TradeOfferManager::builder(String::from(""), "./assets")
        .identity_secret(String::from("secret"))
        .build();
    
    manager.set_cookies(&cookies);
    
    // Passing in GetUserDetailsMethod::None assumes we are friends with the user.
    let user_details = manager.get_user_details(steamid, GetUserDetailsMethod::None).await?;
    // Passing a tradeofferid will convert it into GetUserDetailsMethod::TradeOfferId(tradeofferid)
    // let user_details = manager.get_user_details(&steamid, 5746598837).await?;
    // Passing an access token will convert it into GetUserDetailsMethod::Token(token)
    // let user_details = manager.get_user_details(&steamid, "itfRpc6r").await?;
    
    println!("Trade will result in escrow? {}", user_details.has_escrow().nope().bold());
    Ok(())
}

// Not really necessary, this just makes true/false values display in a more human way.
trait Nope {
    fn nope(&self) -> &'static str;
}

impl Nope for bool {
    fn nope(&self) -> &'static str {
        if *self {
            "yep"
        } else {
            "nope"
        }
    }
}