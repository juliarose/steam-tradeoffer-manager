use crate::SteamID;
use crate::enums::Language;
use crate::types::{AppId, ContextId, HttpClient};
use crate::helpers::DEFAULT_CLIENT;

/// Options for loading a user's inventory.
#[derive(Debug, Clone)]
pub struct GetInventoryOptions<'a> {
    /// Client to use for making requests.
    pub client: &'a HttpClient,
    /// The user's Steam ID.
    pub steamid: SteamID,
    /// App ID of inventory.
    pub appid: AppId,
    /// Context ID of inventory.
    pub contextid: ContextId,
    /// Whether to fetch only tradable items.
    pub tradable_only: bool,
    /// The language to use for descriptions.
    pub language: Language,
    /// The number of items to fetch per page. Defaults to 2000.
    pub page_size: u32,
    /// Optional access token for authenticated requests.
    pub access_token: Option<String>,
}

impl Default for GetInventoryOptions<'_> {
    fn default() -> Self {
        Self {
            client: &DEFAULT_CLIENT,
            steamid: SteamID::default(),
            appid: 0,
            contextid: 0,
            tradable_only: true,
            language: Language::English,
            page_size: 2000,
            access_token: None,
        }
    }
}

impl<'a> GetInventoryOptions<'a> {
    /// Creates a new [`GetInventoryOptions`]. `tradable_only` will be set to `true` and
    /// `language` will be set to [`Language::English`].
    pub fn new(
        steamid: SteamID,
        appid: AppId,
        contextid: ContextId,
    ) -> GetInventoryOptions<'a> {
        Self {
            client: &DEFAULT_CLIENT,
            steamid,
            appid,
            contextid,
            tradable_only: true,
            language: Language::English,
            page_size: 2000,
            access_token: None,
        }
    }
}
