use serde::{Serialize, Deserialize};
use crate::{types::TradeOfferId, serializers::string};

/// The result returned after sending a new trade offer.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SentOffer {
    #[serde(with = "string")]
    pub tradeofferid: TradeOfferId,
    #[serde(default)]
    pub needs_mobile_confirmation: bool,
    #[serde(default)]
    pub needs_email_confirmation: bool,
    pub email_domain: Option<String>,
}