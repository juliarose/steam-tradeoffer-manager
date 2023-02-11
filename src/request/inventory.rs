use crate::{
    SteamID,
    enums::Language,
    types::{AppId, ContextId},
    internal_types::Client,
    helpers::DEFAULT_CLIENT,
};

/// Options for loading a user's inventory.
#[derive(Debug, Clone)]
pub struct GetInventoryOptions<'a> {
    /// Client to use for making requests.
    pub client: &'a Client,
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
        }
    }
}