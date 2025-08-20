
/// Session data from cookies.
#[derive(Debug, Clone, Default)]
pub struct Session {
    /// The session ID.
    pub sessionid: String,
    /// The access token for trade offers.
    pub access_token: String,
    /// The Steam ID of the user.
    pub steamid: u64,
}
