use std::fmt;
use reqwest;
use reqwest_middleware;
use anyhow;
use serde_qs;
use reqwest::StatusCode;
use crate::response::{
    AppId,
    ClassId,
    InstanceId
};

pub const RESPONSE_UNSUCCESSFUL_MESSAGE: &str = "Empty response";

#[derive(Debug)]
pub struct MissingClassInfoError {
    pub appid: AppId,
    pub classid: ClassId,
    pub instanceid: InstanceId,
}

impl fmt::Display for MissingClassInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing description for {}:{}:{:?})", self.appid, self.classid, self.instanceid)
    }
}

impl std::error::Error for MissingClassInfoError {}

#[derive(Debug)]
pub enum APIError {
    ParameterError(&'static str),
    ResponseError(String),
    ReqwestError(reqwest::Error),
    ReqwestMiddlewareError(anyhow::Error),
    StatusError(reqwest::StatusCode),
    QueryParameterError(serde_qs::Error),
    ParseError(serde_json::Error),
    HttpError(StatusCode),
    NotLoggedIn,
    TradeError(String),
    MissingClassInfo(MissingClassInfoError),
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            APIError::ParameterError(s) => write!(f, "{}", s),
            APIError::ResponseError(s) => write!(f, "{}", s),
            APIError::ReqwestError(e) => write!(f, "{}", e),
            APIError::ReqwestMiddlewareError(e) => write!(f, "{}", e),
            APIError::StatusError(e) => write!(f, "{}", e),
            APIError::QueryParameterError(e) => write!(f, "{}", e),
            APIError::ParseError(e) => write!(f, "{}", e),
            APIError::HttpError(e) => write!(f, "{}", e),
            APIError::NotLoggedIn => write!(f, "Not logged in"),
            APIError::TradeError(e) => write!(f, "{}", e),
            APIError::MissingClassInfo(e) => write!(f, "{}", e),
        }
    }
}

impl From<reqwest_middleware::Error> for APIError {
    fn from(error: reqwest_middleware::Error) -> APIError {
        match error {
            reqwest_middleware::Error::Reqwest(e) => {
                APIError::ReqwestError(e)
            },
            reqwest_middleware::Error::Middleware(e) => {
                APIError::ReqwestMiddlewareError(e)
            },
        }
    }
}

impl From<MissingClassInfoError> for APIError {
    fn from(error: MissingClassInfoError) -> APIError {
        APIError::MissingClassInfo(error)
    }
}

impl From<serde_json::Error> for APIError {
    fn from(error: serde_json::Error) -> APIError {
        APIError::ParseError(error)
    }
}

impl From<serde_qs::Error> for APIError {
    fn from(error: serde_qs::Error) -> APIError {
        APIError::QueryParameterError(error)
    }
}

impl From<reqwest::Error> for APIError {
    fn from(error: reqwest::Error) -> APIError {
        APIError::ReqwestError(error)
    }
}
