use crate::types::{AppId, ClassId, InstanceId, TradeOfferId};
use reqwest_middleware;
use std::{fmt, num::ParseIntError, time::SystemTimeError};

/// Any range of errors encountered when making requests.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A paramter is missing or invalid.
    #[error("Invalid parameter: {}", .0)]
    Parameter(&'static str),
    /// An unexpected response containing a message was received. Check the message for more 
    /// details.
    #[error("Unexpected response: {}", .0)]
    Response(String),
    /// An error was encountered during the request.
    #[error("Request error: {}", .0)]
    Reqwest(#[from] reqwest::Error),
    /// An error was encountered within the request middleware.
    #[error("Request middleware error: {}", .0)]
    ReqwestMiddleware(anyhow::Error),
    /// An error was encountered converting parameters to a valid URL string.
    #[error("Unable to convert to query parameters: {}", .0)]
    QueryParameter(#[from] serde_qs::Error),
    /// An error was encountered parsing a JSON response body.
    #[error("Error parsing response: {}", .0)]
    Parse(#[from] serde_json::Error),
    /// An error was encountered on response. This is usually a response with an HTTP code other 
    /// than 200.
    #[error("Error {}", .0.status())]
    Http(reqwest::Response),
    /// You are not logged in.
    #[error("Not logged in")]
    NotLoggedIn,
    /// A response returned a JSON response where `success` is `false`.
    #[error("Response unsuccessful")]
    ResponseUnsuccessful,
    /// An HTML document could not be parsed from the response.
    #[error("Error parsing HTML document: {}", .0)]
    Html(#[from] ParseHtmlError),
    /// An error was encountered when sending or acting on trade offers.
    #[error("Trade error: {}", .0)]
    Trade(TradeOfferError),
    #[error("{}", .0)]
    /// A [ClassInfo] is missing. For some reason a classinfo could not be obtained from Steam or 
    /// the file system. This is rare but can sometimes occur if Steam's servers are having 
    /// issues.
    MissingClassInfo(#[from] MissingClassInfoError),
    /// This trade offer has no confirmations.
    #[error("No confirmation for offer {}", .0)]
    NoConfirmationForOffer(TradeOfferId),
    /// A poll was called within 1 second from the last poll.
    #[error("Poll called too soon after last poll")]
    PollCalledTooSoon,
    /// A number could not be decoded from base64. This means your identity_secret was used and is 
    /// not valid a valid base64 number.
    #[error("Invalid base64: {}", .0)]
    Base64Decode(#[from] base64::DecodeError),
    /// A confirmation could be confirmed.
    #[error("Confirmation unsuccessful. The confirmation may have actually succeeded, the confirmation no longer exist, or another trade may be going through. Check confirmations again to verify.")]
    ConfirmationUnsuccessful,
    /// The response is not expected. The containing string provides a message with more details.
    #[error("Malformed response")]
    MalformedResponse,
    /// An action was taken that depended on polling be setup.
    #[error("No action was taken because polling is not setup.")]
    PollingNotSetup,
    /// An action resulted in the buffer going over its limit.
    #[error("Failed to enqueue action. The polling buffer is full. A maximum of 10 messages can be queued at a time.")]
    PollingBufferFull,
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
    // An error occurred joining reads.
    #[error("Join error")]
    JoinError,
    // A path could not be converted to a string.
    #[error("Path conversion to string failed")]
    PathError,
    #[error("System time failure: {}", .0)]
    SystemTime(SystemTimeError),
}

/// An error received from a response when sending or acting of trade offers.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TradeOfferError {
    /// An unknown error occurred. The contained string will contain additional information.
    #[error("{}", .0)]
    Unknown(String),
    /// An unknown error occurred with a numeric EResult code.
    #[error("{}", .0)]
    UnknownEResult(i32),
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
    /// can't trade (e.g. due to a VAC ban).
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
}

impl TradeOfferError {
    pub fn from_code(code: i32) -> Self {
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
    
    pub fn code(&self) -> Option<i32> {
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
        if let Some(code) = message.trim().split(' ').rev().next() {
            let mut chars = code.chars();
            
            if chars.next() != Some('(') {
                return Self::Unknown(message.into());
            }
            
            if chars.next_back() != Some(')') {
                return Self::Unknown(message.into());
            }
            
            if let Ok(code) = chars.as_str().parse::<i32>() {
                return Self::from_code(code);
            }
        }
        
        Self::Unknown(message.into())
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

/// Details the missing classinfo.
#[derive(thiserror::Error, Debug)]
pub struct MissingClassInfoError {
    pub appid: AppId,
    pub classid: ClassId,
    pub instanceid: InstanceId,
}

impl fmt::Display for MissingClassInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Missing description for {}:{}:{:?})",
            self.appid, self.classid, self.instanceid.unwrap_or(0),
        )
    }
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
    ParseInt(#[from] ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parses_trade_offer_error() {
        let message = "There was an error accepting this trade offer. Please try again later. (28)";
        let error = TradeOfferError::from(message);
        
        assert_eq!(error, TradeOfferError::AlreadyRedeemed);
    }
}