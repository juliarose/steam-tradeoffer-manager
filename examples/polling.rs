use steam_tradeoffer_manager::TradeOfferManager;
use steam_tradeoffer_manager::response::{TradeOffer, Asset};
use steam_tradeoffer_manager::enums::TradeOfferState;
use steam_tradeoffer_manager::error::Error;
use steam_tradeoffer_manager::polling::PollOptions;
use chrono::Duration;
use owo_colors::OwoColorize;

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
    
    println!("{} Active", offer.bright_magenta().bold());
    println!("Receiving: {}", assets_item_names(&offer.items_to_receive));
    println!("Giving: {}", assets_item_names(&offer.items_to_give));
    
    // We're giving something.
    if !offer.items_to_give.is_empty() {
        println!("This offer is not giving us free items - skipping");
        return;
    }
    
    println!("{}", "This offer is giving us free items - accepting".bright_blue());
    
    // Free items.
    if let Err(error) = accept_offer(manager, offer).await {
        println!("Error accepting offer {offer}: {error}");
    } else {
        println!("{} Accepted", offer.bright_magenta().bold());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let cookies = std::env::var("COOKIES")?
        .split("; ")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let api_key = TradeOfferManager::get_api_key(&cookies).await?;
    let manager = TradeOfferManager::builder()
        .api_key(api_key)
        .identity_secret(String::from("secret"))
        .cookies(cookies) // Cookies can also be set using the `set_cookies` method on the manager
        .build();
    
    // Fails if you did not set your cookies.
    let (_tx, mut rx) = manager.start_polling(PollOptions {
        // By default PollOptions does not have a cancel duration.
        cancel_duration: Some(Duration::try_minutes(30).unwrap()),
        ..PollOptions::default()
    })?;
    
    // Listen to the receiver for events.
    while let Some(message) = rx.recv().await {
        match message {
            Ok(offers) => {
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
