use super::response as api_response;
use super::SteamTradeOfferAPI;
use crate::error::{MissingClassInfoError, ParseHtmlError, ParameterError};
use crate::SteamID;
use crate::types::ClassInfoMap;
use crate::response::{self, User, UserDetails};
use std::sync::Arc;
use lazy_regex::Regex;
use lazy_regex::regex_captures;

pub fn offer_referer_url(
    pathname: &str,
    partner: SteamID,
    token: &Option<&str>,
) -> Result<String, ParameterError> {
    let mut params = vec![
        ("partner", partner.account_id().to_string()),
    ];
    
    if let Some(token) = token {
        params.push(("token", token.to_string()));
    }
    
    let url = SteamTradeOfferAPI::get_url(&format!("/tradeoffer/{pathname}"));
    let url = reqwest::Url::parse_with_params(&url, &params)
        .map_err(ParameterError::UrlParse)?;
    
    Ok(url.into())
}

pub fn from_raw_receipt_asset(
    asset: api_response::RawReceiptAsset,
    map: &ClassInfoMap,
) -> Result<response::Asset, MissingClassInfoError> {
    map.get(&(asset.appid, asset.classid, asset.instanceid))
        .map(|classinfo| response::Asset {
            appid: asset.appid,
            contextid: asset.contextid,
            assetid: asset.assetid,
            amount: asset.amount,
            missing: false,
            classinfo: Arc::clone(classinfo),
        })
        .ok_or(MissingClassInfoError {
            appid: asset.appid,
            classid: asset.classid,
            instanceid: asset.instanceid,
        })
}

pub fn parse_user_details(
    body: &str
) -> Result<UserDetails, ParseHtmlError> {
    fn get_days(group: Option<(&str, &str)>) -> u32 {
        match group {
            Some((_, days_str)) => {
                match days_str.parse::<u32>() {
                    Ok(days) => days,
                    Err(_e) => 0,
                }
            },
            None => 0,
        }
    }
    
    // fn get_persona_names(contents: &str) -> Result<(String, String), ParseHtmlError> {
    //     let my_persona_name = regex_captures!(r#"var g_strYourPersonaName = "(?:[^"\\]|\\.)*";\n"#, contents)
    //         .map(|(_, name)| unescape(name))
    //         .flatten()
    //         .ok_or_else(|| ParseHtmlError::Malformed("Missing persona name for me"))?;
    //     let them_persona_name = regex_captures!(r#"var g_strTradePartnerPersonaName = "(.*)";\n"#, contents)
    //         .map(|(_, name)| unescape(name))
    //         .flatten()
    //         .ok_or_else(|| ParseHtmlError::Malformed("Missing persona name for them"))?;
        
    //     Ok((my_persona_name, them_persona_name))
    // }
    
    if let Some((_, _contents)) = regex_captures!(r#"\n\W*<script type="text/javascript">\W*\r?\n?(\W*var g_rgAppContextData[\s\S]*)</script>"#, body) {
        let my_escrow_days = get_days(
            regex_captures!(r#"var g_daysMyEscrow = (\d+);"#, body)
        );
        let them_escrow_days = get_days(
            regex_captures!(r#"var g_daysTheirEscrow = (\d+);"#, body)
        );
        
        Ok(UserDetails {
            me: User {
                escrow_days: my_escrow_days,
            },
            them: User {
                escrow_days: them_escrow_days,
            }
        })
    } else {
        Err(ParseHtmlError::Malformed("Missing script tag"))
    }
}

pub fn parse_receipt_script(
    script: &str,
) -> Result<Vec<api_response::RawReceiptAsset>, ParseHtmlError> {
    Regex::new(r#"oItem\s*=\s*(\{.*\});\s*\n"#)
        .map_err(|_| ParseHtmlError::Malformed("Invalid regexp"))?
        .captures_iter(script)
        // filter out the matches that can't be parsed (e.g. if there are too many digits to store in an i64).
        .map(|capture| if let Some(m) = capture.get(1) {
            let asset = serde_json::from_str::<api_response::RawReceiptAsset>(m.as_str())?;
            
            Ok(asset)
        } else {
            Err(ParseHtmlError::Malformed("Missing capture group in match"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parses_receipt_script_correctly() {
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
    
    #[test]
    fn parses_user_details() {
        let body = include_str!("fixtures/new_offer.html");
        let user_details = parse_user_details(body);
        
        assert!(user_details.is_ok());
    }
    
    #[test]
    fn gets_offer_referer_url() {
        let url = offer_referer_url(
            "new",
            SteamID::from(76561198000000000), 
            &Some("token"),
        ).unwrap();
        
        assert_eq!(url, "https://steamcommunity.com/tradeoffer/new?partner=39734272&token=token");
        
        let url = offer_referer_url(
            "new",
            SteamID::from(76561198000000000), 
            &None,
        ).unwrap();
        
        assert_eq!(url, "https://steamcommunity.com/tradeoffer/new?partner=39734272");
    }
}