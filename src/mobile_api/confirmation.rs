
/// A mobile confirmation. Used primarily for confirming trade offers or listing 
/// items on the market.
#[derive(Debug, PartialEq, Clone)]
pub struct Confirmation {
    /// The ID of the confirmation.
    pub id: u64,
    /// The key of the confirmation.
    pub key: u64,
    /// Trade offer ID or market transaction ID.
    pub creator: u64,
    /// The confirmation type.
    pub conf_type: ConfirmationType,
    /// The description of the confirmation.
    pub description: String,
}

impl Confirmation {
    /// Human readable representation of this confirmation.
    pub fn description(&self) -> String {
        format!("{:?} - {}", self.conf_type, self.description)
    }
}

/// The type of confirmation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConfirmationType {
    Generic = 1,
    Trade = 2,
    MarketSell = 3,
    AccountRecovery = 6,
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