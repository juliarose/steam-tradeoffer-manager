use steam_tradeoffer_manager::{
    SteamID,
    TradeOfferManager,
    response::{TradeOffer, Asset},
    enums::TradeOfferState,
    error::Error,
    polling::PollOptions,
    chrono::Duration,
};

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
    offer: &mut TradeOffer,
) {
    fn assets_item_names<'a>(
        assets: &'a Vec<Asset>,
    ) -> Vec<&'a str> {
        assets
            .iter()
            .map(|item| item.classinfo.market_hash_name.as_ref())
            .collect()
    }
    
    println!("New offer {offer}");
    println!("Receiving: {:?}", assets_item_names(&offer.items_to_receive));
    println!("Giving: {:?}", assets_item_names(&offer.items_to_give));
    
    // free items
    if offer.items_to_give.is_empty() {
        if let Err(error) = accept_offer(manager, offer).await {
            println!("Error accepting offer {offer}: {error}");
        } else {
            println!("Accepted offer {offer}");
        }
    }
}

#[tokio::main]
async fn main() {
    let data_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let (steamid, api_key, cookies) = get_session();
    let manager = TradeOfferManager::builder(
        steamid,
        api_key,
        data_directory,
    )
        .identity_secret(String::from("secret"))
        .build();
    let mut options = PollOptions::default();
    
    // Cookies are required to interact with trade offers.
    manager.set_cookies(&cookies);
    // By default PollOptions does not have a cancel duration.
    options.cancel_duration = Some(Duration::minutes(30));
    
    let mut rx = manager.start_polling(options);
    
    // Listen to the receiver for events.
    while let Some(message) = rx.recv().await {
        match message {
            Ok(offers) => {
                for (mut offer, old_state) in offers {
                    if let Some(state) = old_state {
                        println!(
                            "Offer {} changed state: {} -> {}",
                            offer,
                            state,
                            offer.trade_offer_state
                        );
                    }
                    
                    // This isn't our offer.
                    if !offer.is_our_offer {
                        continue;
                    }
                    
                    if offer.trade_offer_state == TradeOfferState::Active {
                        handle_offer(&manager, &mut offer).await;
                    }
                }
            },
            Err(error) => {
                // If an error occurred during the poll.
                println!("Error encountered polling offers: {error}");
            },
        }
    }
}

/// Gets session from environment variable.
fn get_session() -> (SteamID, String, Vec<String>) {
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    let sid_str = std::env::var("STEAMID")
        .unwrap_or_else(|_| panic!("STEAMID missing"));
    let steamid = SteamID::from(sid_str.parse::<u64>().unwrap());
    let cookies = std::env::var("COOKIES")
        .expect("COOKIES missing")
        .split('&')
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    (steamid, api_key, cookies)
}