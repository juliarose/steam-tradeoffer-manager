use serde::{Serialize, Deserialize};
use crate::serializers::string;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SentOffer {
    #[serde(with = "string")]
    pub tradeofferid: u64,
    #[serde(default)]
    pub needs_mobile_confirmation: bool,
    #[serde(default)]
    pub needs_email_confirmation: bool,
    pub email_domain: Option<String>,
}