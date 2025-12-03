//! Error types.

use crate::enums::TradeOfferState;
use crate::types::*;

use std::num::ParseIntError;

pub use another_steam_totp::Error as TOTPError;
pub use anyhow::Error as AnyhowError;
pub use reqwest::Error as ReqwestError;

/// Result type returned by most methods in this crate.
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Any range of errors encountered when making requests.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An input parameter is missing or invalid.
    #[error("Invalid parameter: {}", .0)]
    Parameter(#[from] ParameterError),
    /// An unexpected response containing a message was received. Check the message for more
    /// details.
    #[error("Unexpected response: {}", .0)]
    UnexpectedResponse(String),
    /// An error was encountered making a request.
    #[error("reqwest error: {}", .0)]
    Reqwest(#[from] ReqwestError),
    /// An error was encountered within the request middleware.
    #[error("reqwest middleware error: {}", .0)]
    ReqwestMiddleware(AnyhowError),
    /// An error was encountered parsing a JSON response body.
    #[error("Error parsing response: {}", .0)]
    ParseJson(#[from] serde_json::Error),
    /// An error was encountered on response. This is a response with an HTTP code other than 200.
    #[error("Error {}", .0)]
    StatusCode(reqwest::StatusCode),
    /// You are not logged in.
    #[error("Not logged in")]
    NotLoggedIn,
    /// A response returned a JSON response where `success` is `false`.
    #[error("Response unsuccessful")]
    ResponseUnsuccessful,
    /// An HTML document could not be parsed from the response.
    #[error("Error parsing HTML document: {}", .0)]
    ParseHtml(#[from] ParseHtmlError),
    /// An error was encountered when sending or acting on trade offers.
    #[error("Trade error: {}", .0)]
    TradeOffer(TradeOfferError),
    /// A classinfo is missing. For some reason a classinfo could not be obtained from Steam or
    /// the file system. This usually shouldn't occur.
    #[error("{}", .0)]
    MissingClassInfo(#[from] MissingClassInfoError),
    /// An error occurred within Steam TOTP.
    #[error("{}", .0)]
    TOTP(#[from] TOTPError),
    /// This trade offer has no confirmations.
    #[error("No confirmation for offer {}", .0)]
    NoConfirmationForOffer(TradeOfferId),
    /// A confirmation could not be confirmed. If a message was contained in the response body it
    /// will be included.
    #[error(
        "Confirmation unsuccessful. {}",
        .0.as_ref().map(|s| s.as_str()).unwrap_or(
            "The confirmation may have succeeded, the confirmation no longer exists, \
or another trade may be going through. Check confirmations again to verify."
        )
    )]
    ConfirmationUnsuccessful(Option<String>),
    /// The response is not expected. Check the contained message for more details.
    #[error("Malformed response: {}", .0)]
    MalformedResponse(&'static str),
    /// The response is not expected. Check the contained message for more details.
    #[error("Malformed response: {}\nRaw body:{}", .0, .1)]
    MalformedResponseWithBody(&'static str, String),
    /// A response from Steam returned an EResult code.
    #[error("Steam EResult error: {}\nRaw body:{}", .0, .1)]
    SteamEResult(u32, String),
}

/// Any number of issues with a provided parameter.
#[derive(thiserror::Error, Debug)]
pub enum ParameterError {
    /// An API key or JWT access token was expected but none was provided.
    #[error("No API key or access token provided. Make sure your API key or cookies are set.")]
    MissingApiKeyOrAccessToken,
    /// No identity secret.
    #[error("No identity secret.")]
    NoIdentitySecret,
    /// Offer is missing trade ID.
    #[error(
        "Offer is missing trade ID. This usually means the offer it belongs to has not yet been \
accepted."
    )]
    MissingTradeId,
    /// Offer is empty.
    #[error("Offer is empty.")]
    EmptyOffer,
    /// Offer is not in accepted state.
    #[error("Offer is not in accepted state. Offer state: {}", .0)]
    NotInAcceptedState(TradeOfferState),
    /// Cannot accept an offer that is not active.
    #[error("Cannot accept an offer that is not active. Offer state: {}", .0)]
    CannotAcceptOfferThatIsNotActive(TradeOfferState),
    /// Cannot accept an offer that we created.
    #[error("Cannot accept an offer that we created.")]
    CannotAcceptOfferWeCreated,
    /// Cannot cancel an offer we did not create.
    #[error("Cannot cancel an offer we did not create.")]
    CannotCancelOfferWeDidNotCreate,
    /// Cannot decline an offer we created.
    #[error("Cannot decline an offer we created.")]
    CannotDeclineOfferWeCreated,
    /// An error was encountered parsing a URL.
    #[error("Unable to parse URL: {}", .0)]
    UrlParse(#[from] url::ParseError),
}

/// An error occurred when working with the file system.
#[derive(thiserror::Error, Debug)]
pub enum FileError {
    /// A generic error.
    #[error("Filesystem error: {}", .0)]
    FileSystem(#[from] std::io::Error),
    /// File contents could not be parsed as JSON.
    #[error("Error parsing file contents: {}", .0)]
    Parse(#[from] serde_json::Error),
    /// A path could not be converted to a string.
    #[error("Path conversion to string failed")]
    PathError,
    /// Error with system time.
    #[error("System time failure: {}", .0)]
    SystemTime(#[from] std::time::SystemTimeError),
}

/// An error occurred when setting cookies.
#[derive(thiserror::Error, Debug)]
pub enum SetCookiesError {
    /// The Steam ID is missing from the cookies.
    #[error("Missing Steam login cookie")]
    MissingSteamLogin,
    /// The Steam ID is missing from the cookies.
    #[error("Access token not found in steamLoginSecure cookie.")]
    MissingAccessToken,
    /// The Steam ID is invalid.
    #[error("Invalid Steam ID: {}", .0)]
    InvalidSteamID(ParseIntError),
}

/// An error received from a response when sending or acting of trade offers.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TradeOfferError {
    /// An unknown error occurred. The contained string will contain additional information.
    #[error("{}", .0)]
    Unknown(String),
    /// An unknown error occurred with a numeric EResult code.
    #[error("Unknown EResult ({})", .0)]
    UnknownEResult(u32),
    /// # Code 2
    /// Returned when a more specific error code couldn't be determined.
    #[error("Fail")]
    Fail,
    /// # Code 11
    /// This trade offer is in an invalid state, and cannot be acted upon. Usually
    /// you'll need to send a new trade offer.
    #[error("InvalidState")]
    InvalidState,
    /// # Code 15
    /// You can't send or accept this trade offer because either you can't trade with the
    /// other user, or one of the parties in this trade can't send or receive one of the
    /// items in the trade.
    /// 
    /// Possible causes:
    /// - You aren't friends with the other user and you didn't provide a trade token.
    /// - The provided trade token was wrong.
    /// - You are trying to send or receive an item for a game in which you or the other user
    ///   can't trade (e.g. due to a VAC ban).
    /// - You are trying to send an item and the other user's inventory is full for that game.
    #[error("AccessDenied")]
    AccessDenied,
    /// # Code 16
    /// The Steam Community web server did not receive a timely reply from the trade  offers
    /// server while sending/accepting this trade offer. It is possible (and not unlikely) 
    /// that the operation actually succeeded.
    #[error("Timeout")]
    Timeout,
    /// # Code 20
    /// As the name suggests, the trade offers service is currently unavailable.
    #[error("ServiceUnavailable")]
    ServiceUnavailable,
    /// # Code 25
    /// Sending this trade offer would put you over your limit. You are limited to 5 Active offers
    /// (including those requiring confirmation, but excluding those in escrow) to a single
    /// recipient, or 30 Active offers total. If you are accepting a trade offer, then your
    /// inventory for a particular game may be full.
    #[error("LimitExceeded")]
    LimitExceeded,
    /// # Code 26
    /// This response code suggests that one or more of the items in this trade offer does not
    /// exist in the inventory from which it was requested.
    #[error("Revoked")]
    Revoked,
    /// # Code 28
    /// When accepting a trade offer, this response code suggests that it has already been
    /// accepted.
    #[error("AlreadyRedeemed")]
    AlreadyRedeemed,
    /// We cannot trade with partner because they have a trade ban.
    #[error("TradeBan")]
    TradeBan,
    /// We have logged in from a new device and temporarily cannot trade.
    #[error("NewDevice")]
    NewDevice,
    /// Partner cannot trade for some reason.
    #[error("PartnerCannotTrade")]
    PartnerCannotTrade,
}

impl TradeOfferError {
    /// Transforms the code number into the corresponding error.
    pub fn from_code(code: u32) -> Self {
        match code {
            2 => Self::Fail,
            11 => Self::InvalidState,
            15 => Self::AccessDenied,
            16 => Self::Timeout,
            20 => Self::ServiceUnavailable,
            25 => Self::LimitExceeded,
            26 => Self::Revoked,
            28 => Self::AlreadyRedeemed,
            _ => Self::UnknownEResult(code),
        }
    }
    
    /// Gets the code number for this error.
    pub fn code(&self) -> Option<u32> {
        match self {
            Self::Fail => Some(2),
            Self::InvalidState => Some(11),
            Self::AccessDenied => Some(15),
            Self::Timeout => Some(16),
            Self::ServiceUnavailable => Some(20),
            Self::LimitExceeded => Some(25),
            Self::Revoked => Some(26),
            Self::AlreadyRedeemed => Some(28),
            Self::UnknownEResult(code) => Some(*code),
            _ => None,
        }
    }
}

impl From<&str> for TradeOfferError {
    fn from(message: &str) -> Self {
        // This simply checks the last piece for a number in parentheses.
        // 
        // Strings generally appear as:
        // "There was an error accepting this trade offer. Please try again later. (28)"
        if let Some(code) = message.trim().split(' ').next_back() {
            let mut chars = code.chars();
            
            if chars.next() != Some('(') {
                return Self::Unknown(message.into());
            }
            
            if chars.next_back() != Some(')') {
                return Self::Unknown(message.into());
            }
            
            // Consume the rest of the characters and attempt to parse it as a number.
            if let Ok(code) = chars.as_str().parse::<u32>() {
                return Self::from_code(code);
            }
        }
        
        Self::Unknown(message.into())
    }
}

impl From<u32> for TradeOfferError {
    /// Converts a u32 error code into a TradeOfferError.
    fn from(code: u32) -> Self {
        Self::from_code(code)
    }
}

impl From<reqwest_middleware::Error> for Error {
    fn from(error: reqwest_middleware::Error) -> Error {
        match error {
            reqwest_middleware::Error::Reqwest(e) => Error::Reqwest(e),
            reqwest_middleware::Error::Middleware(e) => Error::ReqwestMiddleware(e),
        }
    }
}

/// Details of the missing classinfo.
#[derive(thiserror::Error, Debug)]
#[error("Missing classinfo for {}:{}:{}", .appid, .classid, .instanceid.unwrap_or(0))]
pub struct MissingClassInfoError {
    /// The app ID of the missing classinfo.
    pub appid: AppId,
    /// The class ID of the missing classinfo.
    pub classid: ClassId,
    /// The instance ID of the missing classinfo.
    pub instanceid: InstanceId,
}

/// An error occurred when parsing HTML.
#[derive(thiserror::Error, Debug)]
pub enum ParseHtmlError {
    /// The HTML is not what is expected.
    #[error("{}", .0)]
    Malformed(&'static str),
    /// There was an error in the response.
    #[error("{}", .0)]
    Response(String),
    /// An error occurred parsing an integer in the response.
    #[error("{}", .0)]
    ParseInt(#[from] std::num::ParseIntError),
    /// An error occurred parsing JSON in the response.
    #[error("{}", .0)]
    ParseJson(#[from] serde_json::Error),
    /// A selector could not be parsed.
    #[error("Invalid selector")]
    ParseSelector,
}

/// An asset for an item into a trade failed to be converted into its acquired item.
#[derive(thiserror::Error, Debug)]
#[error(
    "Failed to convert item {}:{}:{} into acquired item as it is missing either the \
new_contextid or new_assetid property. This usually means the trade it belongs to has not yet \
been completed.",
    .appid,
    .contextid,
    .assetid
)]
pub struct TryIntoNewAssetError {
    /// App ID.
    pub appid: AppId,
    /// Context ID.
    pub contextid: ContextId,
    /// Asset ID.
    pub assetid: AssetId,
    /// Amount.
    pub amount: Amount,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parses_trade_offer_error() {
        let message = "There was an error accepting this trade offer. \
Please try again later. (28)";
        let error = TradeOfferError::from(message);
        
        assert_eq!(error, TradeOfferError::AlreadyRedeemed);
    }
}
