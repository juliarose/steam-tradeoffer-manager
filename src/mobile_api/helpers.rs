use crate::{error::ParseHtmlError, response::Confirmation, enums::ConfirmationType};
use scraper::{Html, Selector, element_ref::ElementRef};

pub fn parse_confirmations(text: String) -> Result<Vec<Confirmation>, ParseHtmlError> {
    fn parse_description(
        element: ElementRef,
        description_selector: &Selector,
    ) -> Result<Confirmation, ParseHtmlError> {
        let description = element.select(description_selector).next()
            .ok_or(ParseHtmlError::Malformed("Description is missing from confirmation"))?;
        let data_type = element.value().attr("data-type")
            .ok_or(ParseHtmlError::Malformed("Description is missing data-type attribute"))?;
        let id = element.value().attr("data-confid")
            .ok_or(ParseHtmlError::Malformed("Description is missing data-confid attribute"))?;
        let key = element.value().attr("data-key")
            .ok_or(ParseHtmlError::Malformed("Description is missing data-key attribute"))?;
        let creator = element.value().attr("data-creator")
            .ok_or(ParseHtmlError::Malformed("Description is missing data-creator attribute"))?;
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
        .map_err(|_e| ParseHtmlError::ParseSelector)?;
    let mobileconf_done_selector = Selector::parse(".mobileconf_done")
        .map_err(|_e| ParseHtmlError::ParseSelector)?;
    let div_selector = Selector::parse("div")
        .map_err(|_e| ParseHtmlError::ParseSelector)?;
    
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
        .map_err(|_e| ParseHtmlError::ParseSelector)?;
    let description_selector = Selector::parse(".mobileconf_list_entry_description")
        .map_err(|_e| ParseHtmlError::ParseSelector)?;
    let confirmations = fragment.select(&confirmation_list_selector)
        .map(|description| parse_description(description, &description_selector))
        .collect::<Result<Vec<Confirmation>, ParseHtmlError>>()?;
    
    Ok(confirmations)
}