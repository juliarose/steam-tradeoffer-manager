use super::{NewTradeOfferItem, NewTradeOffer};
use crate::SteamID;
use crate::helpers::COMMUNITY_HOSTNAME;

/// Builder for constructing new trade offers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewTradeOfferBuilder {
    /// The partner's [`SteamID`] for this offer.
    pub(crate) partner: SteamID,
    /// The items to give in this offer.
    pub(crate) items_to_give: Vec<NewTradeOfferItem>,
    /// The items to received in this offer.
    pub(crate) items_to_receive: Vec<NewTradeOfferItem>,
    /// The message to send in this offer.
    pub(crate) message: Option<String>,
    /// The access token for sending an offer if you are not friends with the partner.
    pub(crate) token: Option<String>,
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
        self.into()
    }
}
        
fn parse_offer_access_token(trade_offer_url: &str) -> Option<String> {
    let url = url::Url::parse(trade_offer_url).ok()?;
    let hostname = url.host_str();
    
    if hostname != Some(COMMUNITY_HOSTNAME) {
        return None;
    }
    
    url.query_pairs()
        .find(|(key, value)| {
            *key == std::borrow::Cow::Borrowed("token") &&
            // tokens are 8 characters
            value.len() == 8
        })
        .map(|(_, token)| token.to_string())
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
    
    #[test]
    fn none_when_hostname_is_wrong() {
        let url = "https://stemcommunity.com/tradeoffer/new/?partner=0&token=TkA5KFkh";
        
        assert!(parse_offer_access_token(url).is_none());
    }
    
    #[test]
    fn none_when_token_is_missing() {
        let url = "https://stemcommunity.com/tradeoffer/new/?partner=0&token=";
        
        assert!(parse_offer_access_token(url).is_none());
    }
}