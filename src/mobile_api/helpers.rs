use super::confirmation::{Confirmation, ConfirmationType};
use crate::{SteamID, time, error::ParseHtmlError};
use hmacsha1::hmac_sha1;
use sha1::{Sha1, Digest};
use lazy_regex::regex_replace_all;
use scraper::{Html, Selector, element_ref::ElementRef};

const MALFORMED_CONTENT: &str = "Unexpected content format";
const MALFORMED_DESCRIPTION: &str = "Unexpected description format";

pub fn generate_confirmation_hash_for_time(
    time: i64,
    tag: &str,
    identity_secret: &String,
) -> Result<String, base64::DecodeError> {
    let decode: &[u8] = &base64::decode(identity_secret)?;
    let time_bytes = time.to_be_bytes();
    let tag_bytes = tag.as_bytes();
    let array = [&time_bytes, tag_bytes].concat();
    let hash = hmac_sha1(decode, &array);
    
    Ok(base64::encode(hash))
}

pub fn get_device_id(steamid: &SteamID) -> String {
    let mut hasher = Sha1::new();

    hasher.update(u64::from(*steamid).to_string().as_bytes());
    
    let result = hasher.finalize();
    let hash = result.iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let device_id = regex_replace_all!(
        r#"^([0-9a-f]{8})([0-9a-f]{4})([0-9a-f]{4})([0-9a-f]{4})([0-9a-f]{12}).*$"#i,
        &hash,
        |_, a, b, c, d, e| format!("{a}-{b}-{c}-{d}-{e}"),
    );
    
    format!("android:{device_id}")
}

pub fn parse_confirmations(text: String) -> Result<Vec<Confirmation>, ParseHtmlError> {
    fn parse_description(element: ElementRef, description_selector: &Selector) -> Result<Confirmation, ParseHtmlError> {
        let description = element.select(description_selector).next()
            .ok_or(ParseHtmlError::Malformed(MALFORMED_DESCRIPTION))?;
        let data_type = element.value().attr("data-type")
            .ok_or(ParseHtmlError::Malformed(MALFORMED_DESCRIPTION))?;
        let id = element.value().attr("data-confid")
            .ok_or(ParseHtmlError::Malformed(MALFORMED_DESCRIPTION))?;
        let key = element.value().attr("data-key")
            .ok_or(ParseHtmlError::Malformed(MALFORMED_DESCRIPTION))?;
        let creator = element.value().attr("data-creator")
            .ok_or(ParseHtmlError::Malformed(MALFORMED_DESCRIPTION))?;
        let description = description
            .text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let conf_type = data_type
            .try_into()
            .unwrap_or(ConfirmationType::Unknown);
        
        Ok(Confirmation {
            id: id.parse::<u64>()?,
            key: key.parse::<u64>()?,
            conf_type,
            description,
            creator: creator.parse::<u64>()?,
        })
    }

    let fragment = Html::parse_fragment(&text);
    // these should probably never fail
    let mobileconf_empty_selector = Selector::parse("#mobileconf_empty")
        .map_err(|_error| ParseHtmlError::Malformed(MALFORMED_CONTENT))?;
    let mobileconf_done_selector = Selector::parse(".mobileconf_done")
        .map_err(|_error| ParseHtmlError::Malformed(MALFORMED_CONTENT))?;
    let div_selector = Selector::parse("div")
        .map_err(|_error| ParseHtmlError::Malformed(MALFORMED_CONTENT))?;
    
    if let Some(element) = fragment.select(&mobileconf_empty_selector).next() {
        if mobileconf_done_selector.matches(&element) {
            if let Some(element) = element.select(&div_selector).nth(1) {
                let error_message = element
                    .text()
                    .collect::<String>();
                
                return Err(ParseHtmlError::Response(error_message));
            } else {
                return Ok(Vec::new());
            }
        } else {
            return Ok(Vec::new());
        }
    }
    
    let confirmation_list_selector = Selector::parse(".mobileconf_list_entry")
        .map_err(|_error| ParseHtmlError::Malformed(MALFORMED_CONTENT))?;
    let description_selector = Selector::parse(".mobileconf_list_entry_description")
        .map_err(|_error| ParseHtmlError::Malformed(MALFORMED_CONTENT))?;
    let confirmations = fragment.select(&confirmation_list_selector)
        .map(|description| parse_description(description, &description_selector))
        .collect::<Result<Vec<Confirmation>, ParseHtmlError>>()?;
    
    Ok(confirmations)
}

pub fn server_time(time_offset: i64) -> i64 {
    time::get_system_time() as i64 + time_offset
}