use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, EnumString};

/// Filter for getting trade offers.
#[derive(Serialize_repr, Deserialize_repr, Display, EnumString, Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum OfferFilter {
    /// Fetch active offers only.
    ActiveOnly = 1,
    /// Fetch historical offers only.
    HistoricalOnly = 2,
    /// Fetch all offers.
    All = 3,
}
