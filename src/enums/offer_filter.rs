use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde_repr::{Serialize_repr, Deserialize_repr};
use strum_macros::{Display, EnumString};

#[derive(Serialize_repr, Deserialize_repr, Display, EnumString, Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum OfferFilter {
	ActiveOnly = 1,
	HistoricalOnly = 2,
	All = 3
}