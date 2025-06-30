use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde_repr::{Serialize_repr, Deserialize_repr};
use strum_macros::{Display, EnumString};

/// The state of a trade offer.
#[derive(Debug, Serialize_repr, Deserialize_repr, Display, EnumString, PartialEq, TryFromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum TradeOfferState {
    /// Invalid.
    Invalid = 1,
    /// This trade offer has been sent, neither party has acted on it yet.
    Active = 2,
    /// The trade offer was accepted by the recipient and items were exchanged.
    Accepted = 3,
    /// The recipient made a counter offer.
    Countered = 4,
    /// The trade offer was not accepted before the expiration date.
    Expired = 5,
    /// The sender cancelled the offer.
    Canceled = 6,
    /// The recipient declined the offer.
    Declined = 7,
    /// Some of the items in the offer are no longer available (indicated by the missing flag in 
	/// the output).
    InvalidItems = 8,
    /// The offer hasn't been sent yet and is awaiting email/mobile confirmation. The offer is 
	/// only visible to the sender.
    CreatedNeedsConfirmation = 9,
    /// Either party canceled the offer via email/mobile. The offer is visible to both parties, 
	/// even if the sender canceled it before it was sent.
    CanceledBySecondFactor = 10,
    /// The trade has been placed on hold. The items involved in the trade have all been removed 
	/// from both parties' inventories and will be automatically delivered in the future.
    InEscrow = 11,
}
