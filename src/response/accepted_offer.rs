use serde::{Serialize, Deserialize};

/// The result returned after accepting a trade offer.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AcceptedOffer {
    #[serde(default)]
    /// Whether the offer needs to be confirmed on mobile or not.
    pub needs_mobile_confirmation: bool,
    #[serde(default)]
    /// Whether the offer needs to be confirmed by email or not.
    pub needs_email_confirmation: bool,
    /// The email domain for this account.
    pub email_domain: Option<String>,
}