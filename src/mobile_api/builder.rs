use super::MobileAPI;
use crate::helpers::USER_AGENT_STRING;
use std::sync::Arc;
use reqwest::cookie::Jar;
use reqwest_middleware::ClientWithMiddleware;

/// Builder for constructing a [`MobileAPI`].
#[derive(Debug, Clone)]
pub struct MobileAPIBuilder {
    /// The identity secret for the account (optional). Required for mobile confirmations.
    pub(crate) identity_secret: Option<String>,
    /// Request cookies.
    pub(crate) cookies: Option<Arc<Jar>>,
    /// Client to use for requests. Remember to also include the cookies connected to this client.
    pub(crate) client: Option<ClientWithMiddleware>,
    /// User agent for requests.
    pub(crate) user_agent: &'static str,
    /// How many seconds your computer is behind Steam's servers. Used in mobile confirmations.
    pub(crate) time_offset: i64,
}

impl Default for MobileAPIBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MobileAPIBuilder {
    /// Creates a new [`MobileAPIBuilder`].
    pub fn new() -> Self {
        Self {
            identity_secret: None,
            cookies: None,
            client: None,
            user_agent: USER_AGENT_STRING,
            time_offset: 0,
        }
    }
    
    /// The identity secret for the account. Required for mobile confirmations.
    pub fn identity_secret(mut self, identity_secret: String) -> Self {
        self.identity_secret = Some(identity_secret);
        self
    }
    
    /// Client to use for requests. It is also required to include the associated cookies with this
    /// client so that the `set_cookies` method works as expected.
    pub fn client(mut self, client: ClientWithMiddleware, cookies: Arc<Jar>) -> Self {
        self.client = Some(client);
        self.cookies = Some(cookies);
        self
    }
    
    /// How many seconds your computer is behind Steam's servers. Used in mobile confirmations.
    pub fn time_offset(mut self, time_offset: i64) -> Self {
        self.time_offset = time_offset;
        self
    }
    
    /// Builds the [`MobileAPI`].
    pub fn build(self) -> MobileAPI {
        self.into()
    }
}