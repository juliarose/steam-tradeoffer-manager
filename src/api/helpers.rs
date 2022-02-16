use super::raw;
use crate::{
    classinfo_cache::ClassInfoCache,
    MissingClassInfoError,
    response,
};
use steamid_ng::SteamID;

pub fn from_raw_trade_offer(offer: raw::RawTradeOffer, cache: &mut ClassInfoCache) -> Result<response::trade_offer::TradeOffer, MissingClassInfoError> {
    fn collect_items(assets: Vec<raw::RawAsset>, cache: &mut ClassInfoCache) -> Result<Vec<response::asset::Asset>, MissingClassInfoError> {
        let mut items = Vec::new();
        
        for asset in assets {
            if let Some(classinfo) = cache.get_classinfo(&(asset.appid, asset.classid, asset.instanceid)) {
                items.push(response::asset::Asset {
                    classinfo,
                    appid: asset.appid,
                    contextid: asset.contextid,
                    assetid: asset.assetid,
                    amount: asset.amount,
                });
            } else {
                // todo use a less broad error for this
                return Err(MissingClassInfoError {
                    appid: asset.appid,
                    classid: asset.classid,
                    instanceid: asset.instanceid,
                });
            }
        }
        
        Ok(items)
    }
    
    fn steamid_from_accountid(accountid: u32) -> SteamID {
        SteamID::new(
            accountid,
            steamid_ng::Instance::Desktop,
            steamid_ng::AccountType::Individual,
            steamid_ng::Universe::Public
        )
    }
    
    let items_to_give = collect_items(offer.items_to_give, cache)?;
    let items_to_receive = collect_items(offer.items_to_receive, cache)?;
    
    Ok(response::trade_offer::TradeOffer {
        items_to_give,
        items_to_receive,
        tradeofferid: offer.tradeofferid,
        trade_offer_state: offer.trade_offer_state,
        partner: steamid_from_accountid(offer.accountid_other),
        message: offer.message,
        is_our_offer: offer.is_our_offer,
        from_real_time_trade: offer.from_real_time_trade,
        expiration_time: offer.expiration_time,
        time_updated: offer.time_updated,
        time_created: offer.time_created,
        escrow_end_date: offer.escrow_end_date,
        confirmation_method: offer.confirmation_method,
    })
}