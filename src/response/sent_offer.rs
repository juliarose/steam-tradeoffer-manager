use serde::{Serialize, Deserialize};
use crate::{types::TradeOfferId, serialize::string};

/// The result returned after sending a new trade offer.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SentOffer {
    /// The ID of the offer sent.
    #[serde(with = "string")]
    pub tradeofferid: TradeOfferId,
    #[serde(default)]
    /// Whether the offer needs mobile confirmation or not.
    pub needs_mobile_confirmation: bool,
    #[serde(default)]
    /// Whether the offer needs email confirmation or not.
    pub needs_email_confirmation: bool,
    /// The email domain if this offer requires email confirmation.
    pub email_domain: Option<String>,
}

impl SentOffer {
    /// Whether the offer needs to be confirmed by mobile or email.
    pub fn needs_confirimation(&self) -> bool {
        self.needs_mobile_confirmation || self.needs_email_confirmation
    }
}