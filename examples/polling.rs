use steam_tradeoffer_manager::{
    TradeOfferManager,
    response::{TradeOffer, Asset},
    enums::TradeOfferState,
    error::Error,
    SteamID,
    chrono::Duration,
};
use dotenv::dotenv;
use std::env;

fn assets_item_names<'a>(
    assets: &'a Vec<Asset>,
) -> Vec<&'a str> {
    assets
        .iter()
        .map(|item| item.classinfo.market_hash_name.as_ref())
        .collect::<Vec<_>>()
}

async fn accept_offer(
    manager: &TradeOfferManager,
    offer: &mut TradeOffer,
) -> Result<(), Error> {
    let accepted_offer = manager.accept_offer(offer).await?;
    
    if accepted_offer.needs_mobile_confirmation {
        manager.confirm_offer(offer).await
    } else {
        Ok(())
    }
}

async fn handle_offer(
    manager: &TradeOfferManager,
    mut offer: &mut TradeOffer,
) {
    println!("New offer {}", offer);
    println!("Receiving: {:?}", assets_item_names(&offer.items_to_receive));
    println!("Giving: {:?}", assets_item_names(&offer.items_to_give));
    
    // free items
    if offer.items_to_give.is_empty() {
        if let Err(error) = accept_offer(&manager, &mut offer).await {
            println!("Error accepting offer {}: {}", offer, error);
        } else {
            println!("Accepted offer {}", offer);
        }
    }
}

/// Gets session from environment variable.
fn get_session() -> (String, Vec<String>) {
    let mut sessionid = None;
    let mut cookies: Vec<String> = Vec::new();
    let cookies_str = env::var("COOKIES")
        .expect("COOKIES missing");
    
    for cookie in cookies_str.split('&') {
        let mut split = cookie.split('=');
        
        if split.next().unwrap() == "sessionid" {
            sessionid = Some(split.next().unwrap().to_string());
        }
        
        cookies.push(cookie.to_string());
    }
    
    (sessionid.unwrap(), cookies)
}

/// Gets steamid from environment variable.
fn get_steamid(key: &str) -> SteamID {
    let sid_str = env::var(key)
        .unwrap_or_else(|_| panic!("{} missing", key));
    
    SteamID::from(sid_str.parse::<u64>().unwrap())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let steamid = get_steamid("STEAMID");
    let key = env::var("API_KEY").expect("API_KEY missing");
    let manager = TradeOfferManager::builder(steamid, key)
        .identity_secret(String::from("secret"))
        .cancel_duration(Duration::minutes(30))
        .build();
    let (sessionid, cookies) = get_session();
    
    manager.set_session(&sessionid, &cookies).expect("Could not set session");
    
    // gets changes to trade offers for account
    loop {
        match manager.do_poll(true, true).await {
            Ok(offers) => {
                for (mut offer, old_state) in offers {
                    if let Some(state) = old_state {
                        println!(
                            "Offer {} changed state: {} -> {}",
                            offer,
                            state,
                            offer.trade_offer_state
                        );
                    } else if
                        offer.trade_offer_state == TradeOfferState::Active &&
                        !offer.is_our_offer
                    {
                        handle_offer(&manager, &mut offer).await;
                    }
                }
            },
            Err(error) => {
                println!("Error polling offers: {}", error);
            },
        }
        
        // poll every 30 seconds
        // use async_std sleep if you need a non-blocking sleep
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}