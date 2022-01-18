use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UserDetails {
    pub them_escrow: u32,
    pub my_escrow: u32,
}