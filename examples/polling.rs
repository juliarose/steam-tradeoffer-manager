use steam_tradeoffer_manager::{
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
    fn assets_item_names(assets: &Vec<Asset>) -> Vec<&str> {
        assets.iter().map(|item| item.classinfo.market_name.as_ref()).collect()
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
async fn main() -> Result<(), Error> {
    let (api_key, cookies) = get_session();
    let manager = TradeOfferManager::builder(api_key, "./assets")
        .identity_secret(String::from("secret"))
        .build();
    let options = PollOptions {
        // By default PollOptions does not have a cancel duration.
        cancel_duration: Some(Duration::minutes(30)),
        ..PollOptions::default()
    };
    
    // Cookies are required before starting polling.
    manager.set_cookies(&cookies);
    
    // Fails if you did not set your cookies.
    let mut rx = manager.start_polling(options)?;
    
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
    
    Ok(())
}

/// Gets session from environment variable.
fn get_session() -> (String, Vec<String>) {
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("API_KEY").expect("API_KEY missing");
    let cookies = std::env::var("COOKIES").expect("COOKIES missing")
        .split('&')
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    (api_key, cookies)
}