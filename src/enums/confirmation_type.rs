use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde_repr::{Serialize_repr, Deserialize_repr};
use strum_macros::Display;

/// The type of confirmation.
#[derive(Debug, Serialize_repr, Deserialize_repr, Display, PartialEq, TryFromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum ConfirmationType {
    /// Generic.
    Generic = 1,
    /// Confirmation to confirm trade.
    Trade = 2,
    /// Confirmation to confirm on market.
    MarketSell = 3,
    /// Confirmation for account recovery.
    AccountRecovery = 6,
    /// Uknnown.
    Unknown,
}

impl From<&str> for ConfirmationType {
    fn from(text: &str) -> Self {
        match text {
            "1" => ConfirmationType::Generic,
            "2" => ConfirmationType::Trade,
            "3" => ConfirmationType::MarketSell,
            "6" => ConfirmationType::AccountRecovery,
            _ => ConfirmationType::Unknown,
        }
    }
}