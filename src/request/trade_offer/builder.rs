use super::{NewTradeOfferItem, NewTradeOffer};
use crate::SteamID;

/// Builder for constructing new trade offers.
pub struct NewTradeOfferBuilder {
    /// The partner's [`SteamID`] for this offer.
    pub partner: SteamID,
    /// The items to give in this offer.
    pub items_to_give: Vec<NewTradeOfferItem>,
    /// The items to received in this offer.
    pub items_to_receive: Vec<NewTradeOfferItem>,
    /// The message to send in this offer.
    pub message: Option<String>,
    /// The access token for sending an offer if you are not friends with the partner.
    pub token: Option<String>,
}

impl NewTradeOfferBuilder {
    /// Creates a new [`NewTradeOfferBuilder`] with the given partner.
    pub fn new(partner: SteamID) -> Self {
        Self {
            partner,
            items_to_give: Vec::new(),
            items_to_receive: Vec::new(),
            message: None,
            token: None,
        }
    }
    
    /// The items to give in this offer.
    pub fn items_to_give<T>(mut self, items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<NewTradeOfferItem>
    {
        self.items_to_give = items.into_iter().map(|i| i.into()).collect();
        self
    }
    
    /// The items to received in this offer.
    pub fn items_to_receive<T>(mut self, items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<NewTradeOfferItem>
    {
        self.items_to_receive = items.into_iter().map(|i| i.into()).collect();
        self
    }
    
    /// The trade offer URL for sending an offer if you are not friends with the partner. 
    /// Silently fails if the URL does not contain a token. If you want to check if the token
    /// was parsed successfully check if the `token` of the builder is `Some`.
    pub fn trade_offer_url(mut self, trade_offer_url: &str) -> Self {
        self.token = parse_offer_access_token(trade_offer_url);
        self
    }
    
    /// The token for sending an offer if you are not friends with the partner.
    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }
    
    /// The message to send in this offer.
    pub fn message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
    
    /// Builds into [`NewTradeOffer`].
    pub fn build(self) -> NewTradeOffer {
        NewTradeOffer {
            partner: self.partner,
            items_to_give: self.items_to_give,
            items_to_receive: self.items_to_receive,
            message: self.message,
            token: self.token,
        }
    }
}
        
fn parse_offer_access_token(trade_offer_url: &str) -> Option<String> {
    if let Ok(url) = url::Url::parse(trade_offer_url) {
        let pairs = url.query_pairs();
        let hostname = url.host_str();
        
        if hostname != Some("steamcommunity.com") {
            return None;
        }
        
        for (key, value) in pairs {
            if key == std::borrow::Cow::Borrowed("token") {
                if value.len() == 8 {
                    return Some(value.to_string());
                } else {
                    // not a valid token
                    return None;
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parses_trade_offer_url() {
        let url = "https://steamcommunity.com/tradeoffer/new/?partner=0&token=TkA5KFkh";
        let token = parse_offer_access_token(url).unwrap();
        
        assert_eq!(token, "TkA5KFkh");
    }
}