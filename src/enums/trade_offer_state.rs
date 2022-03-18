use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde_repr::{Serialize_repr, Deserialize_repr};
use strum_macros::{Display, EnumString};

#[derive(Serialize_repr, Deserialize_repr, Display, EnumString, Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum TradeOfferState {
    Invalid = 1,
	Active = 2,
	Accepted = 3,
	Countered = 4,
	Expired = 5,
	Canceled = 6,
	Declined = 7,
	InvalidItems = 8,
	CreatedNeedsConfirmation = 9,
	CanceledBySecondFactor = 10,
	InEscrow = 11,
}