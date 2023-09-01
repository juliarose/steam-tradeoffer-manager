/// Method for obtaining a user's escrow details.
#[derive(Debug, Clone)]
pub enum GetUserDetailsMethod {
    /// Obtain details without using an identifier. This will only work if you are friends with 
    /// the user.
    None,
    /// Obtain details using an access token.
    Token(String),
    /// Obtain details using a tradeofferid.
    TradeOfferId(u64),
}

impl GetUserDetailsMethod {
    /// The token to use for the request.
    pub fn token(&self) -> Option<&str> {
        match self {
            Self::Token(s) => Some(s.as_str()),
            _ => None,
        }
    }
    
    /// The pathname to use for the request.
    pub fn pathname(&self) -> String {
        match self {
            Self::None |
            Self::Token(_) => "new".into(),
            Self::TradeOfferId(tradeofferid) => tradeofferid.to_string(),
        }
    }
}

impl From<Option<String>> for GetUserDetailsMethod {
    fn from(value: Option<String>) -> Self {
        if let Some(value) = value {
            Self::Token(value)
        } else {
            Self::None
        }
    }
}

impl From<&Option<String>> for GetUserDetailsMethod {
    fn from(value: &Option<String>) -> Self {
        if let Some(value) = value {
            Self::Token(value.clone())
        } else {
            Self::None
        }
    }
}

impl From<u64> for GetUserDetailsMethod {
    fn from(value: u64) -> Self {
        Self::TradeOfferId(value)
    }
}

impl From<&u64> for GetUserDetailsMethod {
    fn from(value: &u64) -> Self {
        Self::TradeOfferId(*value)
    }
}

impl From<String> for GetUserDetailsMethod {
    fn from(value: String) -> Self {
        Self::Token(value)
    }
}

impl From<&str> for GetUserDetailsMethod {
    fn from(value: &str) -> Self {
        Self::Token(value.to_owned())
    }
}