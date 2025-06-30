use serde::{Serialize, Deserialize};
use strum_macros::Display;

/// The type of confirmation.
#[derive(Debug, Serialize, Deserialize, Display, PartialEq, Clone, Copy)]
#[repr(u32)]
#[serde(from = "u32")]
pub enum ConfirmationType {
    /// Generic.
    Generic = 1,
    /// Confirmation to confirm trade.
    Trade = 2,
    /// Confirmation to confirm on market.
    MarketSell = 3,
    /// Confirmation for account recovery.
    AccountRecovery = 6,
    /// Unknown.
	Unknown(u32),
}

impl Default for ConfirmationType {
    fn default() -> Self {
        Self::Generic
    }
}

impl From<u32> for ConfirmationType {
    fn from(text: u32) -> Self {
        match text {
            1 => ConfirmationType::Generic,
            2 => ConfirmationType::Trade,
            3 => ConfirmationType::MarketSell,
            6 => ConfirmationType::AccountRecovery,
            other => ConfirmationType::Unknown(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Deserialize)]
    struct Confirmation {
        conf_type: ConfirmationType,
    }
    
    #[test]
    fn deserializes_trade_confirmation() {
        let json: &str = r#"{"conf_type":2}"#;
        let confirmation: Confirmation = serde_json::from_str(json).unwrap();
        
        assert_eq!(confirmation.conf_type, ConfirmationType::Trade);
    }
    
    #[test]
    fn deserializes_unknown_confirmation_type() {
        let json: &str = r#"{"conf_type":10}"#;
        let confirmation: Confirmation = serde_json::from_str(json).unwrap();
        
        assert_eq!(confirmation.conf_type, ConfirmationType::Unknown(10));
    }
}
