use crate::enums::ConfirmationType;

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