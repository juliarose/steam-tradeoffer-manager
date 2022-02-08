use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AcceptedOffer {
    #[serde(default)]
    pub needs_mobile_confirmation: bool,
    #[serde(default)]
    pub needs_email_confirmation: bool,
    pub email_domain: Option<String>,
}