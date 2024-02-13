use crate::types::Client;
use crate::error::{TradeOfferError, Error};
use std::path::PathBuf;
use std::sync::Arc;
use std::fmt::Write;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest::header;
use reqwest::cookie::{Jar, CookieStore};
use serde::de::DeserializeOwned;
use lazy_regex::{regex_captures, regex_is_match};
use async_fs::File;
use futures::io::AsyncWriteExt;
use lazy_static::lazy_static;
use directories::BaseDirs;

lazy_static! {
    pub static ref DEFAULT_CLIENT: Client = {
        let cookie_store = Arc::new(Jar::default());
        
        get_default_middleware(
            cookie_store,
            USER_AGENT_STRING,
        )
    };
}

pub fn default_data_directory() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.config_dir().join("rust-steam-tradeoffer-manager")
    } else {
        "./rust-steam-tradeoffer-manager".into()
    }
}

/// A browser user agent string.
pub const USER_AGENT_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36";
pub(crate) const COMMUNITY_HOSTNAME: &str = "steamcommunity.com";
pub(crate) const WEB_API_HOSTNAME: &str = "api.steampowered.com";

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
pub fn get_sessionid_and_steamid_from_cookies(
    cookies: &[String],
) -> (Option<String>, Option<u64>) {
    let mut sessionid = None;
    let mut steamid = None;
    
    for cookie in cookies {
        if let Some((_, key, value)) = regex_captures!(r#"([^=]+)=(.+)"#, cookie) {
            match key {
                "sessionid" => sessionid = Some(value.to_string()),
                "steamLogin" |
                "steamLoginSecure" => if let Some((_, steamid_str)) = regex_captures!(r#"^(\d{17})"#, value) {
                    steamid = steamid_str.parse::<u64>().ok();
                },
                _ => {},
            }
        }
    }
    
    (sessionid, steamid)
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
            async_fs::rename(&temp_filepath,&filepath).await?;
            Ok(())
        },
        Err(error) => {
            // something went wrong writing to this file...
            async_fs::remove_file(&temp_filepath).await?;
            Err(error)
        }
    }
}

/// Creates a client middleware which includes a cookie store and user agent string.
pub fn get_default_middleware<T>(
    cookie_store: Arc<T>,
    user_agent_string: &'static str,
) -> ClientWithMiddleware
where
    T: CookieStore + 'static,
{
    let mut headers = header::HeaderMap::new();
    
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static(user_agent_string));
    
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

/// Deserializes and checks response for errors.
pub async fn parses_response<D>(
    response: reqwest::Response,
) -> Result<D, Error>
where
    D: DeserializeOwned,
{
    let status = &response.status();
    let body = match status.as_u16() {
        300..=399 if is_login(response.headers().get("location")) => {
            Err(Error::NotLoggedIn)
        },
        400..=499 => Err(Error::StatusCode(response.status())),
        500..=599 => Err(Error::StatusCode(response.status())),
        _ => Ok(response.bytes().await?),
    }?;
    
    match serde_json::from_slice::<D>(&body) {
        Ok(body) => Ok(body),
        Err(parse_error) => {
            // unexpected response
            let html = String::from_utf8_lossy(&body);
            
            if html.contains(r#"<h1>Sorry!</h1>"#) {
                if let Some((_, message)) = regex_captures!("<h3>(.+)</h3>", &html) {
                    Err(Error::UnexpectedResponse(message.into()))
                } else {
                    Err(Error::MalformedResponse)
                }
            } else if html.contains(r#"<h1>Sign In</h1>"#) && html.contains(r#"g_steamID = false;"#) {
                Err(Error::NotLoggedIn)
            } else if regex_is_match!(r#"\{"success": ?false\}"#, &html) {
                Err(Error::ResponseUnsuccessful)
            } else if let Some((_, message)) = regex_captures!(r#"<div id="error_msg">\s*([^<]+)\s*</div>"#, &html) {
                Err(Error::TradeOffer(TradeOfferError::from(message)))
            } else {
                log::error!("Error parsing body `{}`: {}", parse_error, String::from_utf8_lossy(&body));
                
                Err(Error::Parse(parse_error))
            }
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
}
