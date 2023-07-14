use crate::enums::ConfirmationType;
use crate::ServerTime;
use crate::serialize;
use std::fmt;
use chrono::serde::ts_seconds;
use serde::{Serialize, Deserialize};

/// A mobile confirmation. Used primarily for confirming trade offers or listing 
/// items on the market.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Confirmation {
    /// The ID of the confirmation.
    #[serde(with = "serialize::string")]
    pub id: u64,
    /// Trade offer ID or market transaction ID.
    #[serde(with = "serialize::string")]
    pub creator_id: u64,
    /// The time the confirmation was created.
    #[serde(with = "ts_seconds")]
    pub creation_time: ServerTime,
    /// The nonce.
    #[serde(with = "serialize::string")]
    pub nonce: u64,
    /// The cancel text.
    pub cancel: String,
    /// The accept text e.g. "Accept" or "Send Offer".
    pub accept: String,
    /// `true` if can be confirmed with multiple other confirmations.
    #[serde(default)]
    pub multi: bool,
    /// The confirmation type.
    #[serde(default)]
    pub r#type: ConfirmationType,
    /// The type name.
    pub type_name: String,
    /// The headline.
    pub headline: String,
    /// The description.
    #[serde(default)]
    pub summary: Vec<String>,
    /// The icon.
    #[serde(default)]
    pub icon: Option<String>,
    /// Warnings.
    #[serde(default)]
    pub warn: Option<Vec<String>>,
}

impl fmt::Display for Confirmation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.r#type, self.headline)
    }
}

impl Confirmation {
    /// Description for items we are giving.
    pub fn giving(&self) -> Option<&str> {
        self.summary.first().map(|s| s.as_str())
    }
    
    /// Description for items we are receiving.
    pub fn receiving(&self) -> Option<&str> {
        let mut iter = self.summary.iter();
        // consume first element
        let _ = iter.next();
        
        iter.next().map(|s| s.as_str())
    }
}

mod tests {
    #[test]
    fn parsed_trade_offer_confirmation() {
        let confirmation: super::Confirmation = serde_json::from_str(include_str!("fixtures/confirmation.json")).unwrap();
        
        assert_eq!(confirmation.id, 13799599785);
        assert_eq!(confirmation.nonce, 9141945700999917347);
        assert_eq!(confirmation.r#type, super::ConfirmationType::Trade);
    }
}