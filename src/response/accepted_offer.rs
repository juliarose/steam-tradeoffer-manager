use serde::{Serialize, Deserialize};

/// The result returned after accepting a trade offer.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcceptedOffer {
    /// Whether the offer needs to be confirmed on mobile or not.
    #[serde(default)]
    pub needs_mobile_confirmation: bool,
    /// Whether the offer needs to be confirmed by email or not.
    #[serde(default)]
    pub needs_email_confirmation: bool,
    /// The email domain for this account.
    pub email_domain: Option<String>,
}

impl AcceptedOffer {
    /// Whether the offer needs to be confirmed by mobile or email.
    pub fn needs_confirimation(&self) -> bool {
        self.needs_mobile_confirmation || self.needs_email_confirmation
    }
}
