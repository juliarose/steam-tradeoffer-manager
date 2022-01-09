use serde::{Serialize, Deserialize};
use crate::serializers::string;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SentOffer {
    #[serde(with = "string")]
    tradeofferid: u64,
    pub needs_mobile_confirmation: bool,
    pub needs_email_confirmation: bool,
    pub email_domain: String,
}