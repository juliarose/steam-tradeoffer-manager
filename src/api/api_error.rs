use std::fmt;
use reqwest::{
    self,
    StatusCode
};
use reqwest_middleware;
use anyhow;
use serde_qs;
use crate::types::{
    AppId,
    ClassId,
    InstanceId
};

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
    Parameter(&'static str),
    Response(String),
    Reqwest(reqwest::Error),
    ReqwestMiddleware(anyhow::Error),
    QueryParameter(serde_qs::Error),
    Parse(serde_json::Error),
    Http(StatusCode),
    NotLoggedIn,
    Trade(String),
    MissingClassInfo(MissingClassInfoError),
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            APIError::Parameter(s) => write!(f, "{}", s),
            APIError::Response(s) => write!(f, "{}", s),
            APIError::Reqwest(e) => write!(f, "{}", e),
            APIError::ReqwestMiddleware(e) => write!(f, "{}", e),
            APIError::QueryParameter(e) => write!(f, "{}", e),
            APIError::Parse(e) => write!(f, "{}", e),
            APIError::Http(e) => write!(f, "{}", e),
            APIError::NotLoggedIn => write!(f, "Not logged in"),
            APIError::Trade(e) => write!(f, "{}", e),
            APIError::MissingClassInfo(e) => write!(f, "{}", e),
        }
    }
}

impl From<reqwest_middleware::Error> for APIError {
    fn from(error: reqwest_middleware::Error) -> APIError {
        match error {
            reqwest_middleware::Error::Reqwest(e) => {
                APIError::Reqwest(e)
            },
            reqwest_middleware::Error::Middleware(e) => {
                APIError::ReqwestMiddleware(e)
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
        APIError::Parse(error)
    }
}

impl From<serde_qs::Error> for APIError {
    fn from(error: serde_qs::Error) -> APIError {
        APIError::QueryParameter(error)
    }
}

impl From<reqwest::Error> for APIError {
    fn from(error: reqwest::Error) -> APIError {
        APIError::Reqwest(error)
    }
}
