use std::{fmt, num::ParseIntError};
use crate::types::{AppId, ClassId, InstanceId};
use reqwest_middleware;
use reqwest::{self, StatusCode};

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
    #[error("{}", .0)]
    Http(StatusCode),
    #[error("Not logged in")]
    NotLoggedIn,
    #[error("Error parsing HTML document: {}", .0)]
    Html(#[from] ParseHtmlError),
    #[error("Trade error: {}", .0)]
    Trade(String),
    #[error("{}", .0)]
    MissingClassInfo(#[from] MissingClassInfoError),
}

impl From<reqwest_middleware::Error> for Error {
    fn from(error: reqwest_middleware::Error) -> Error {
        match error {
            reqwest_middleware::Error::Reqwest(e) => {
                Error::Reqwest(e)
            },
            reqwest_middleware::Error::Middleware(e) => {
                Error::ReqwestMiddleware(e)
            },
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
        write!(f, "Missing description for {}:{}:{:?})", self.appid, self.classid, self.instanceid)
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