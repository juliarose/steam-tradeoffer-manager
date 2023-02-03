use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde_repr::{Serialize_repr, Deserialize_repr};
use strum_macros::{Display, EnumString};

/// The method of confirmation.
#[derive(Debug, Serialize_repr, Deserialize_repr, Display, EnumString, PartialEq, TryFromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum ConfirmationMethod {
    /// Invalid.
    None = 0,
    /// An email was sent with details on how to confirm the trade offer.
    Email = 1,
    /// The trade offer may be confirmed via the mobile app.
    MobileApp = 2,
}