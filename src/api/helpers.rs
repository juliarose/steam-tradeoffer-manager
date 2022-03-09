use super::raw;
use lazy_regex::Regex;
use crate::{
    classinfo_cache::ClassInfoCache,
    MissingClassInfoError,
    response,
};
use serde_json;
use steamid_ng::SteamID;

pub fn from_raw_receipt_asset(asset: raw::RawReceiptAsset, cache: &mut ClassInfoCache) -> Result<response::asset::Asset, MissingClassInfoError> {
    if let Some(classinfo) = cache.get_classinfo(&(asset.appid, asset.classid, asset.instanceid)) {
        Ok(response::asset::Asset {
            classinfo,
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
        })
    } else {
        Err(MissingClassInfoError {
            appid: asset.appid,
            classid: asset.classid,
            instanceid: asset.instanceid,
        })
    }  
}

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
        tradeid: offer.tradeid,
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

pub fn parse_receipt_script(script: &str) -> Result<Vec<raw::RawReceiptAsset>, &'static str> {
    let re = Regex::new(r#"oItem *= *(\{.*\}); *\n"#).map_err(|_| "Invalid regexp")?;
    
    re.captures_iter(script)
        // try to parse the string matches as i64 (inferred from fn type signature)
        // and filter out the matches that can't be parsed (e.g. if there are too many digits to store in an i64).
        .map(|capture| {
            if let Some(m) = capture.get(1) {
                if let Ok(asset) = serde_json::from_str::<raw::RawReceiptAsset>(m.as_str()) {
                    Ok(asset)
                } else {
                    Err("Failed to deserialize item")
                }
            } else {
                // shouldn't happen...
                Err("Missing capture group in match")
            }
            
        })
        // collect the results in to a Vec<i64> (inferred from fn type signature)
        .collect::<Result<Vec<_>, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_receipt_script_correctly() {
        let script = r#"
            oItem = {"id":"11292488054","owner":"0","amount":"1","classid":"101785959","instanceid":"11040578","icon_url":"fWFc82js0fmoRAP-qOIPu5THSWqfSmTELLqcUywGkijVjZULUrsm1j-9xgEAaR4uURrwvz0N252yVaDVWrRTno9m4ccG2GNqxlQoZrC2aG9hcVGUWflbX_drrVu5UGki5sAij6tOtQ","icon_url_large":"fWFc82js0fmoRAP-qOIPu5THSWqfSmTELLqcUywGkijVjZULUrsm1j-9xgEAaR4uURrwvz0N252yVaDVWrRTno9m4ccG2GNqxlQoZrC2aG9hcVGUWflbX_drrVu5UGki5sAij6tOtQ","icon_drag_url":"","name":"Mann Co. Supply Crate Key","market_hash_name":"Mann Co. Supply Crate Key","market_name":"Mann Co. Supply Crate Key","name_color":"7D6D00","background_color":"3C352E","type":"Level 5 Tool","tradable":1,"marketable":1,"commodity":1,"market_tradable_restriction":"7","market_marketable_restriction":"0","descriptions":[{"value":"Used to open locked supply crates."},{"value":" "},{"value":"This is a limited use item. Uses: 1","color":"00a000","app_data":{"limited":1}}],"tags":[{"internal_name":"Unique","name":"Unique","category":"Quality","color":"7D6D00","category_name":"Quality"},{"internal_name":"TF_T","name":"Tool","category":"Type","category_name":"Type"}],"app_data":{"limited":1,"quantity":"1","def_index":"5021","quality":"6","filter_data":{"1662615936":{"element_ids":["991457757"]},"931505789":{"element_ids":["991457757","4294967295"]}},"player_class_ids":"","highlight_color":"7a6e65"},"pos":1,"appid":440,"contextid":2};
            oItem.appid = 440;
            oItem.contextid = 2;
            oItem.amount = 1;
            oItem.is_stackable = oItem.amount > 1;
            BuildHover( 'item0', oItem, UserYou );
            $('item0').show();
            oItem = {"id":"11292488061","owner":"0","amount":"1","classid":"101785959","instanceid":"11040578","icon_url":"fWFc82js0fmoRAP-qOIPu5THSWqfSmTELLqcUywGkijVjZULUrsm1j-9xgEAaR4uURrwvz0N252yVaDVWrRTno9m4ccG2GNqxlQoZrC2aG9hcVGUWflbX_drrVu5UGki5sAij6tOtQ","icon_url_large":"fWFc82js0fmoRAP-qOIPu5THSWqfSmTELLqcUywGkijVjZULUrsm1j-9xgEAaR4uURrwvz0N252yVaDVWrRTno9m4ccG2GNqxlQoZrC2aG9hcVGUWflbX_drrVu5UGki5sAij6tOtQ","icon_drag_url":"","name":"Mann Co. Supply Crate Key","market_hash_name":"Mann Co. Supply Crate Key","market_name":"Mann Co. Supply Crate Key","name_color":"7D6D00","background_color":"3C352E","type":"Level 5 Tool","tradable":1,"marketable":1,"commodity":1,"market_tradable_restriction":"7","market_marketable_restriction":"0","descriptions":[{"value":"Used to open locked supply crates."},{"value":" "},{"value":"This is a limited use item. Uses: 1","color":"00a000","app_data":{"limited":1}}],"tags":[{"internal_name":"Unique","name":"Unique","category":"Quality","color":"7D6D00","category_name":"Quality"},{"internal_name":"TF_T","name":"Tool","category":"Type","category_name":"Type"}],"app_data":{"limited":1,"quantity":"1","def_index":"5021","quality":"6","filter_data":{"1662615936":{"element_ids":["991457757"]},"931505789":{"element_ids":["991457757","4294967295"]}},"player_class_ids":"","highlight_color":"7a6e65"},"pos":2,"appid":440,"contextid":2};
            oItem.appid = 440;
            oItem.contextid = 2;
            oItem.amount = 1;
            oItem.is_stackable = oItem.amount > 1;
        "#;
        let scripts = parse_receipt_script(script).unwrap();
        
        assert_eq!(scripts.len(), 2);
    }
}