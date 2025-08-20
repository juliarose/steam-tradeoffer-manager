use serde::{Deserialize, Serialize};
use std::cmp;

/// Details for users.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDetails {
    /// Details about you.
    pub me: User,
    /// Details about them.
    pub them: User,
}

/// Details for a single user.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    /// Their escrow duration in days.
    pub escrow_days: u32,
}

impl UserDetails {
    /// Whether the trade would result in escrow or not.
    pub fn has_escrow(&self) -> bool {
        self.them.escrow_days > 0 || self.me.escrow_days > 0
    }
    
    /// The number of days the trade would be held in escrow.
    pub fn hold_duration_days(&self) -> u32 {
        cmp::max(self.them.escrow_days, self.me.escrow_days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn escrow_works() {
        let details = UserDetails {
            me: User {
                escrow_days: 0,
            },
            them: User {
                escrow_days: 3,
            },
        };

        assert!(details.has_escrow());
    }
    
    #[test]
    fn hold_duration_days_works() {
        let details = UserDetails {
            me: User {
                escrow_days: 0,
            },
            them: User {
                escrow_days: 15,
            },
        };

        assert_eq!(15, details.hold_duration_days());
    }
}
