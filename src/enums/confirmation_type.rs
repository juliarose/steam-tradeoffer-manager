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