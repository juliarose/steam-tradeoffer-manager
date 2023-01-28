use steam_tradeoffer_manager::{
    TradeOfferManager,
    request::{NewTradeOffer, NewTradeOfferItem},
    SteamID,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let (steamid, api_key, sessionid, cookies) = get_session();
    let steamid_other = get_steamid("STEAMID_OTHER");
    let manager = TradeOfferManager::builder(
        steamid,
        api_key,
        data_directory,
    ).build();
    let offer = NewTradeOffer::builder(steamid_other)
        .items_to_receive(vec![
            NewTradeOfferItem {
                appid: 440,
                contextid: 2,
                amount: 1,
                assetid: 11482399896,
            },
        ])
        .build();
        
    manager.set_session(&sessionid, &cookies);
    manager.send_offer(&offer).await?;
    
    Ok(())
}

fn get_steamid(key: &str) -> SteamID {
    let sid_str = std::env::var(key)
        .unwrap_or_else(|_| panic!("{} missing", key));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}

/// Gets session from environment variable.
fn get_session() -> (SteamID, String, String, Vec<String>) {
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    let steamid = get_steamid("STEAMID");
    let mut sessionid = None;
    let mut cookies: Vec<String> = Vec::new();
    let cookies_str = std::env::var("COOKIES")
        .expect("COOKIES missing");
    
    for cookie in cookies_str.split('&') {
        let mut split = cookie.split('=');
        
        if split.next().unwrap() == "sessionid" {
            sessionid = Some(split.next().unwrap().to_string());
        }
        
        cookies.push(cookie.to_string());
    }
    
    (steamid, api_key, sessionid.unwrap(), cookies)
}