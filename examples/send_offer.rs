use steam_tradeoffer_manager::{TradeOfferManager, request::NewTradeOffer, SteamID};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let cookies = std::env::var("COOKIES")?
        .split("; ")
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();
    let steamid: SteamID = std::env::var("STEAMID_OTHER")?.parse::<u64>()?.into();
    // A blank API key can be passed, it isn't needed in this example.
    let manager = TradeOfferManager::new(String::from(""), "./assets");
    
    manager.set_cookies(&cookies);
    
    // This method returns only tradable items.
    let inventory = manager.get_inventory(steamid, 440, 2).await?;
    let items = inventory.into_iter().take(5);
    let offer = NewTradeOffer::builder(steamid)
        // Any items that implement Into<NewTradeOfferItem> are fine.
        .items_to_receive(items)
        .message("ayo the pizza here".into())
        .build();
    // This isn't a full offer, but rather some details about the offer sent such as its 
    // tradeofferid and whether it needs mobile confirmation.
    let sent_offer = manager.send_offer(&offer).await?;
    
    // Since we didn't add any items on our side this doesn't need mobile confirmation.
    if sent_offer.needs_mobile_confirmation {
        // But if it did... 
        manager.confirm_offer_id(sent_offer.tradeofferid).await?;
    }
    
    println!("Offer sent: {sent_offer:?}");
    Ok(())
}