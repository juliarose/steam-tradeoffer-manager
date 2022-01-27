use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UserDetails {
    pub them_escrow: u32,
    pub my_escrow: u32,
}

fn get_max(a: u32, b: u32) -> u32 {
    if a > b {
        a
    } else {
        b
    }
}

impl UserDetails {
    
    pub fn has_escrow(&self) -> bool {
        self.them_escrow > 0 || self.my_escrow > 0
    }
    
    pub fn hold_duration_days(&self) -> u32 {
        get_max(self.them_escrow, self.my_escrow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn escrow_works() {
        let details = UserDetails {
            them_escrow: 0,
            my_escrow: 3,
        };

        assert_eq!(true, details.has_escrow());
    }
    
    #[test]
    fn hold_duration_days_works() {
        let details = UserDetails {
            them_escrow: 0,
            my_escrow: 3,
        };

        assert_eq!(3, details.hold_duration_days());
    }
}