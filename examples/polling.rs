use steam_tradeoffer_manager::{
    TradeOfferManager,
    response::{TradeOffer, Asset},
    enums::TradeOfferState,
    error::Error,
    polling::PollOptions,
    chrono::Duration,
};
use owo_colors::OwoColorize;

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

async fn accept_free_items(
    manager: &TradeOfferManager,
    offer: &mut TradeOffer,
) {
    fn assets_item_names(assets: &[Asset]) -> String {
        assets
            .iter()
            .map(|item| item.classinfo.market_name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
    
    println!("{} Active", offer.bright_magenta().bold());
    println!("Receiving: {}", assets_item_names(&offer.items_to_receive));
    println!("Giving: {}", assets_item_names(&offer.items_to_give));
    
    // We're giving something.
    if !offer.items_to_give.is_empty() {
        println!("This offer is not giving us free items - skipping");
        return;
    }
    
    // Free items.
    if let Err(error) = accept_offer(manager, offer).await {
        println!("Error accepting offer {offer}: {error}");
    } else {
        println!("{} Accepted", offer.bright_magenta().bold());
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    
    let cookies = std::env::var("COOKIES").expect("COOKIES missing")
        .split("; ")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let api_key = TradeOfferManager::get_api_key(&cookies).await?;
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
                if offers.is_empty() {
                    println!("Got poll; no changes");
                }
                
                for (mut offer, old_state) in offers {
                    if let Some(state) = old_state {
                        println!(
                            "{} Offer changed state: {state} -> {}",
                            offer.bright_magenta().bold(),
                            offer.trade_offer_state,
                        );
                    }
                    
                    // Skip offers that are ours.
                    if offer.is_our_offer {
                        continue;
                    }
                    
                    if offer.trade_offer_state == TradeOfferState::Active {
                        accept_free_items(&manager, &mut offer).await;
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