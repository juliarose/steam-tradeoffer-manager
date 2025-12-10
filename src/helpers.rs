// This module is a bit disorganized but contains various utility functions and types. 

use crate::error::{Error, SetCookiesError, TradeOfferError};
use crate::types::HttpClient;
use crate::session::Session;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use async_fs::File;
use bytes::Bytes;
use directories::BaseDirs;
use futures::io::AsyncWriteExt;
use lazy_regex::{regex_captures, regex_is_match};
use lazy_static::lazy_static;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::header;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::de::{self, DeserializeOwned, Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use serde_json::de::SliceRead;

lazy_static! {
    pub static ref DEFAULT_CLIENT: HttpClient = {
        let cookie_store = Arc::new(Jar::default());
        
        get_default_client(cookie_store, USER_AGENT_STRING)
    };
}

/// A browser user agent string.
pub const USER_AGENT_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, \
like Gecko) Chrome/97.0.4692.71 Safari/537.36";
pub(crate) const COMMUNITY_HOSTNAME: &str = "steamcommunity.com";
pub(crate) const WEB_API_HOSTNAME: &str = "api.steampowered.com";

#[derive(Debug, Clone)]
pub struct CookiesData {
    pub sessionid: Option<String>,
    pub steamid: u64,
    pub access_token: String,
}

#[derive(Debug, Default)]
struct TradeErrorOrEResultResponse<'a> {
    num_keys: usize,
    response: Option<&'a str>,
    str_error: Option<&'a str>,
}

pub fn default_data_directory() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.config_dir().join("rust-steam-tradeoffer-manager")
    } else {
        "./rust-steam-tradeoffer-manager".into()
    }
}

/// Generates a random sessionid.
pub fn generate_sessionid() -> String {
    // Should look like "37bf523a24034ec06c60ec61"
    (0..12).fold(String::new(), |mut output, _| { 
        let b = rand::random::<u8>();
        let _ = write!(output, "{b:02x?}");
        
        output
    })
}

/// Extracts the session ID and Steam ID from cookie values.
pub fn extract_auth_data_from_cookies(
    cookies: &[String],
) -> Result<CookiesData, SetCookiesError> {
    let mut sessionid = None;
    let mut steamid = 0;
    let mut access_token = None;
     
    for cookie in cookies {
        if let Some((
            _,
            key,
            value,
        )) = regex_captures!(r#"([^=]+)=(.+)"#, cookie) {
            match key {
                "sessionid" => sessionid = Some(value.to_string()),
                "steamLoginSecure" => if let Some((
                    _,
                    steamid_str,
                    access_token_str,
                )) = regex_captures!(r#"^(\d{17})%7C%7C([^;]+)"#, value) {
                    steamid = steamid_str
                        .parse::<u64>()
                        .map_err(SetCookiesError::InvalidSteamID)?;
                    access_token = Some(access_token_str.to_string());
                } else {
                    return Err(SetCookiesError::MissingAccessToken);
                },
                _ => {},
            }
        }
    }
    
    let access_token = access_token.ok_or(SetCookiesError::MissingAccessToken)?;
    
    Ok(CookiesData {
        sessionid,
        steamid,
        access_token,
    })
}

/// Extracts the session from cookies and returns a [`Session`] object. This will generate a new
/// session ID if one is not found in the cookies. The cookies will be modified to include the new
/// session ID.
pub fn get_session_from_cookies(
    cookies: &mut Vec<String>,
) -> Result<Session, SetCookiesError> {
    let CookiesData {
        sessionid,
        steamid,
        access_token,
    } = extract_auth_data_from_cookies(cookies)?;
    let sessionid = if let Some(sessionid) = sessionid {
        sessionid
    } else {
        // the cookies don't contain a sessionid
        let sessionid = generate_sessionid();
        
        cookies.push(format!("sessionid={sessionid}"));
        sessionid
    };
    let session = Session {
        sessionid,
        access_token,
        steamid,
    };
    
    Ok(session)
}

/// Writes a file atomically.
pub async fn write_file_atomic(
    filepath: PathBuf,
    bytes: &[u8],
) -> std::io::Result<()> {
    let mut temp_filepath = filepath.clone();
    
    temp_filepath.set_extension("tmp");
    
    let mut temp_file = File::create(&temp_filepath).await?;
    
    match temp_file.write_all(bytes).await {
        Ok(_) => {
            temp_file.flush().await?;
            async_fs::rename(&temp_filepath, &filepath).await?;
            Ok(())
        },
        Err(error) => {
            // something went wrong writing to this file...
            // any errors removing the temporary file aren't important
            let _ = async_fs::remove_file(&temp_filepath).await;
            Err(error)
        }
    }
}

/// Creates a client middleware which includes a cookie store and user agent string.
pub fn get_default_client<T>(
    cookie_store: Arc<T>,
    user_agent_string: &'static str,
) -> ClientWithMiddleware
where
    T: CookieStore + 'static,
{
    let mut headers = header::HeaderMap::new();
    
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(user_agent_string),
    );
    
    let client = reqwest::ClientBuilder::new()
        .cookie_provider(cookie_store)
        .default_headers(headers)
        .build()
        .unwrap();
    
    ClientBuilder::new(client)
        .build()
}

/// Checks if location is login.
fn is_login(location_option: Option<&header::HeaderValue>) -> bool {
    if let Some(location) = location_option {
        if let Ok(location_str) = location.to_str() {
            // starts_with is probably more accurate (should be tested)
            return location_str.contains("/login")
        }
    }
    
    false
}

/// Deserializes a response that may contain a `str_error` or an `EResult` code.
/// 
/// This function does not allocate.
fn deserialize_response_for_errors<'a>(
    bytes: &'a Bytes,
) -> Result<TradeErrorOrEResultResponse<'a>, serde_json::Error> {
    // This function is much longer than it could be.
    // Since parsing responses is a frequent operation we probably don't want to double allocate
    // responses just to check for errors.
    struct TradeErrorOrEResultVisitor<'a> {
        marker: std::marker::PhantomData<&'a ()>,
    }

    impl<'de, 'a> Visitor<'de> for TradeErrorOrEResultVisitor<'a>
    where
        'de: 'a,
    {
        type Value = TradeErrorOrEResultResponse<'a>;
        
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a JSON object with optional 'response' and 'strError' fields")
        }
        
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut response = TradeErrorOrEResultResponse::default();

            while let Some(key) = access.next_key::<&str>()? {
                response.num_keys += 1;

                match key {
                    "response" => {
                        response.response = Some(access.next_value()?);
                    }
                    "strError" => {
                        response.str_error = Some(access.next_value()?);
                    }
                    _ => {
                        access.next_value::<de::IgnoredAny>()?;
                    }
                }
            }
            
            Ok(response)
        }
    }
    
    let mut deserializer = serde_json::de::Deserializer::new(SliceRead::new(bytes));
    let response = deserializer.deserialize_any(TradeErrorOrEResultVisitor {
        marker: std::marker::PhantomData
    })?;
    
    Ok(response)
}

/// Checks the response for errors. EResult is the x-eresult header which may include an EResult
/// code.
fn check_response_for_errors(bytes: &Bytes, eresult: Option<u32>) -> Result<(), Error> {
    if let Ok(json) = deserialize_response_for_errors(bytes) {
        // Handle trade errors
        // https://github.com/DoctorMcKay/node-steam-tradeoffer-manager/blob/06b73c50a73d0880154cec816ccb70e660719311/lib/helpers.js#L14
        if let Some(str_error) = json.str_error {
            // Try to extract an eresult code at the end of the message
            let eresult = str_error
                .rsplit_once('(')
                .and_then(|(_, num)| num.strip_suffix(')'))
                .and_then(|num| num.trim().parse::<u32>().ok());
            // Match known error cause strings
            let trade_err = if str_error.contains("You cannot trade with") &&
            str_error.contains("trade ban") {
                TradeOfferError::TradeBan
            } else if str_error.contains("You have logged in from a new device") {
                TradeOfferError::NewDevice
            } else if str_error.contains("is not available to trade") {
                TradeOfferError::PartnerCannotTrade
            } else if str_error.contains("sent too many trade offers") {
                TradeOfferError::LimitExceeded
            } else if str_error.contains("unable to contact the game's item server") {
                TradeOfferError::ServiceUnavailable
            } else if let Some(code) = eresult {
                TradeOfferError::UnknownEResult(code)
            } else {
                TradeOfferError::Unknown(str_error.to_string())
            };
            
            return Err(Error::TradeOffer(trade_err));
        }
        
        if let Some(code) = eresult {
            // Not an error
            if code == 1 || json.num_keys > 1 {
                return Ok(());
            }
            
            if let Some(response) = json.response {
                // Check that this is an object that is not empty.
                // This is probably good enough without needing to deserialize the object
                let response_has_data = {
                    response.starts_with('{') &&
                    response.ends_with('}') &&
                    response != "{}"
                };
                
                if !response_has_data {
                    let body = String::from_utf8_lossy(bytes).into();
                    
                    return Err(Error::SteamEResult(code, body));
                }
            }
        }
    }
        
    Ok(())
}

/// Deserializes and checks response for errors.
pub async fn parses_response<D>(
    response: reqwest::Response,
) -> Result<D, Error>
where
    D: DeserializeOwned,
{
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.bytes().await?;
    // Check x-eresult Steam header
    let eresult = headers
        .get("x-eresult")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok());
    
    // Log non-success status and include body for debugging
    if !status.is_success() {
        let body_text = String::from_utf8_lossy(&bytes);
        log::warn!("Steam response error. Status: {status}, Body: {body_text}");
        
        // Redirects that might imply not logged in
        if (300..=399).contains(&status.as_u16()) {
            if let Some(location) = headers.get("location") {
                if is_login(Some(location)) {
                    return Err(Error::NotLoggedIn);
                }
            }
        }
        
        // Capture general error by status range
        if (400..=599).contains(&status.as_u16()) {
            return Err(Error::StatusCode(status));
        }
    }
    
    // This doesn't return anything but will catch errors in the response.
    check_response_for_errors(&bytes, eresult)?;
    
    match serde_json::from_slice::<D>(&bytes) {
        Ok(body) => Ok(body),
        Err(_) => {
            // unexpected response
            let html = String::from_utf8_lossy(&bytes);
            
            if html.contains(r#"<h1>Sorry!</h1>"#) {
                return if let Some((
                    _,
                    message,
                )) = regex_captures!("<h3>(.+)</h3>", &html) {
                    Err(Error::UnexpectedResponse(message.into()))
                } else {
                    Err(Error::MalformedResponseWithBody(
                        "Steam returned an HTML response but an error message could not be \
                        detected (an <h3> tag was expected but was not found)",
                        html.into()
                    ))
                };
            }
            
            if html.contains(r#"<h1>Sign In</h1>"#) && html.contains(r#"g_steamID = false;"#) {
                return Err(Error::NotLoggedIn);
            }
            
            if regex_is_match!(r#"\{"success": ?false\}"#, &html) {
                return Err(Error::ResponseUnsuccessful);
            }
    
            // Session seems expired
            if html.contains("Access is denied") {
                return Err(Error::NotLoggedIn);
            }
            
            if let Some((
                _,
                message,
            )) = regex_captures!(r#"<div id="error_msg">\s*([^<]+)\s*</div>"#, &html) {
                return Err(Error::TradeOffer(TradeOfferError::from(message)));
            }
            
            Err(Error::MalformedResponseWithBody(
                "Got unexpected non-JSON response.",
                html.into()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn generates_session() {
        let sessionid = generate_sessionid();
        
        assert_eq!(sessionid.len(), 24);
    }
    
    #[test]
    fn deserializes_str_error_response() {
        let json = r#"{
            "strError":"You cannot trade with this user because they have a trade ban (12345)"
        }"#;
        let bytes = Bytes::from(json);
        let result = deserialize_response_for_errors(&bytes).unwrap();
        
        assert_eq!(result.str_error, Some(
            "You cannot trade with this user because they have a trade ban (12345)"
        ));
        assert_eq!(result.response, None);
        assert_eq!(result.num_keys, 1);
    }
    
    #[test]
    fn str_error_response_is_error() {
        let json = r#"{
            "strError":"You cannot trade with this user because they have a trade ban (12345)"
        }"#;
        let bytes = Bytes::from(json);
        let result = check_response_for_errors(&bytes, None);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::TradeOffer(TradeOfferError::TradeBan)));
    }
}
