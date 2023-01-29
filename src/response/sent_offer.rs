use serde::{Serialize, Deserialize};
use crate::{types::TradeOfferId, serializers::string};

/// The result returned after sending a new trade offer.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SentOffer {
    #[serde(with = "string")]
    /// The ID of the offer sent.
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