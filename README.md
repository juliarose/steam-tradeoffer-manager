# steam-tradeoffers

Makes steam trade offers easy.

Still a work in progress as I flesh out and test the APIs.

## Features
- Richly-featured API for creating, accepting, cancelling, and declining trade offers.
- Manages account trade offer state.
- Loading inventories.
- Mobile confirmations.
- Loads descriptions (classinfos) for assets. Classinfos are cached to file and read when available. The manager holds a [Least frequently used (LFU) cache](https://en.wikipedia.org/wiki/Least_frequently_used) of classinfos in memory to reduce file reads.

## Usage
```rs
use steam_tradeoffers::{
    TradeOfferManager,
    Asset,
    TradeOfferState,
    SteamID,
};

fn assets_item_names<'a>(assets: &'a Vec<Asset>) -> Vec<&'a str> {
    assets
        .iter()
        .map(|item| item.classinfo.market_hash_name.as_ref())
        .collect::<Vec<_>>()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let steamid = SteamID::from(0);
    let manager = TradeOfferManager::new(
        &steamid,
        "key",
        Some("secret".into()),
    );
    let sessionid = "sessionid";
    let cookies = vec![String::from("cookie=value")];
    
    manager.set_session(sessionid, &cookies)?;
    
    // gets changes to trade offers for account
    for (offer, old_state) in manager.do_poll(true).await? {
        if let Some(state) = old_state {
            println!(
                "Offer {} changed state: {} -> {}",
                offer,
                state,
                offer.trade_offer_state
            );
        } else if offer.trade_offer_state == TradeOfferState::Active {
            if offer.is_our_offer {
                // we sent this offer
            } else {
                println!(
                    "New offer {}",
                    offer
                );
                println!(
                    "Receiving: {:?}",
                    assets_item_names(&offer.items_to_receive)
                );
                println!(
                    "Giving: {:?}",
                    assets_item_names(&offer.items_to_give)
                );
            }
        }
    }
    
    // loads inventory
    let items = manager.get_inventory(&steamid, 440, 2, true).await?;
    
    println!("{} items in your inventory", items.len());
    
    Ok(())
}
```

## Thanks

Inspired by the excellent [node-steam-tradeoffer-manager](https://github.com/DoctorMcKay/node-steam-tradeoffer-manager) module. Thanks to https://github.com/dyc3/steamguard-cli (steamguard) for functionality relating to mobile confirmations.

## LICENSE

MIT