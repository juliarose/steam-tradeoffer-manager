use std::cmp;
use serde::{Serialize, Deserialize};

/// Details for user.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct UserDetails {
    /// Their escrow duration in days.
    pub them_escrow_days: u32,
    /// Your escrow duration in days.
    pub my_escrow_days: u32,
}

impl UserDetails {
    /// Whether the trade would result in escrow or not.
    pub fn has_escrow(&self) -> bool {
        self.them_escrow_days > 0 || self.my_escrow_days > 0
    }
    
    /// The number of days the trade would be held in escrow.
    pub fn hold_duration_days(&self) -> u32 {
        cmp::max(self.them_escrow_days, self.my_escrow_days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn escrow_works() {
        let details = UserDetails {
            them_escrow_days: 0,
            my_escrow_days: 3,
        };

        assert_eq!(true, details.has_escrow());
    }
    
    #[test]
    fn hold_duration_days_works() {
        let details = UserDetails {
            them_escrow_days: 0,
            my_escrow_days: 15,
        };

        assert_eq!(15, details.hold_duration_days());
    }
}