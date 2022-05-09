use crate::types::{AppId, ClassId, InstanceId, TradeOfferId};
use reqwest_middleware;
use std::{fmt, num::ParseIntError};

#[derive(thiserror::Error, Debug)]
pub enum FileError {
    #[error("Filesystem error: {}", .0)]
    FileSystem(#[from] std::io::Error),
    #[error("Error parsing file contents: {}", .0)]
    Parse(#[from] serde_json::Error),
    #[error("Join error")]
    JoinError,
    #[error("Path conversion to string failed")]
    PathError,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid parameter: {}", .0)]
    Parameter(&'static str),
    #[error("Unexpected response: {}", .0)]
    Response(String),
    #[error("Request error: {}", .0)]
    Reqwest(#[from] reqwest::Error),
    #[error("Request middleware error: {}", .0)]
    ReqwestMiddleware(anyhow::Error),
    #[error("Unable to convert to query parameters: {}", .0)]
    QueryParameter(#[from] serde_qs::Error),
    #[error("Error parsing response: {}", .0)]
    Parse(#[from] serde_json::Error),
    #[error("Error {}", .0.status())]
    Http(reqwest::Response),
    #[error("Not logged in")]
    NotLoggedIn,
    #[error("Error parsing HTML document: {}", .0)]
    Html(#[from] ParseHtmlError),
    #[error("Trade error: {}", .0)]
    Trade(TradeOfferError),
    #[error("{}", .0)]
    MissingClassInfo(#[from] MissingClassInfoError),
    #[error("No confirmation for offer {}", .0)]
    NoConfirmationForOffer(TradeOfferId),
}

#[derive(thiserror::Error, Debug, PartialEq)]
#[repr(u8)]
pub enum TradeOfferError {
    #[error("{}", .0)]
    Unknown(String),
    #[error("{}", .0)]
    UnknownEResult(i32),
    #[error("Fail")]
    Fail,
    #[error("InvalidState")]
    InvalidState,
    #[error("AccessDenied")]
    AccessDenied,
    #[error("Timeout")]
    Timeout,
    #[error("ServiceUnavailable")]
    ServiceUnavailable,
    #[error("TimeLimitExceededout")]
    LimitExceeded,
    #[error("Revoked")]
    Revoked,
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
            26 => Self::LimitExceeded,
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
            Self::AlreadyRedeemed => Some(2),
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
            self.appid, self.classid, self.instanceid
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseHtmlError {
    #[error("{}", .0)]
    Malformed(&'static str),
    #[error("{}", .0)]
    Response(String),
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