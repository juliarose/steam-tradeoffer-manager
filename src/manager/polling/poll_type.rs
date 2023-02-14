use crate::ServerTime;

/// The type of poll to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollType {
    /// Let the manager decide. Unless you need to fetch offers in special cases this is what 
    /// should be used.
    Auto,
    /// Fastest method for obtaining new offers. This will fetch only active offers and includes 
    /// descriptions in the response rather than relying on ISteamEconomy/GetAssetClassInfo. 
    /// For this reason, items in the response will also not contain app_data. This will not update 
    /// the timestamps in the poll data. For this reason, this should not be used as your only 
    /// method of polling if you care about checking the state of changed offers.
    NewOffers,
    /// Do a full update.
    FullUpdate,
    /// Performs a poll fetching offers since the given time.
    OffersSince(ServerTime),
}

impl PollType {
    /// The poll is a full update.
    pub(crate) fn is_full_update(&self) -> bool {
        matches!(self, Self::FullUpdate)
    }
    
    /// The poll is only active offers.
    pub(crate) fn is_active_only(&self) -> bool {
        matches!(self, Self::NewOffers)
    }
}