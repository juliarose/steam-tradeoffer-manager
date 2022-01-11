use reqwest::header::HeaderValue;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest::{header, cookie::CookieStore};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use crate::APIError;
use lazy_regex::{regex_is_match, regex_captures};

const USER_AGENT_STRING: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36";

pub fn get_default_middleware<T>(cookie_store: Arc<T>) -> ClientWithMiddleware
where
    T: CookieStore + 'static
{
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let mut headers = header::HeaderMap::new();
    
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static(USER_AGENT_STRING));
    
    let client = reqwest::ClientBuilder::new()
        .cookie_provider(cookie_store)
        .default_headers(headers)
        .build()
        .unwrap();
    
    ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

fn is_login(location_option: Option<&HeaderValue>) -> bool {
    match location_option {
        Some(location) => {
            if let Ok(location_str) = location.to_str() {
                regex_is_match!("/login", location_str)
            } else {
                false
            }
        },
        None => false,
    }
}

pub async fn parses_response<D>(response: reqwest::Response) -> Result<D, APIError>
where
    D: DeserializeOwned
{
    let status = &response.status();
    
    match status.as_u16() {
        300..=399 if is_login(response.headers().get("location")) => {
            Err(APIError::NotLoggedIn)
        },
        400..=499 => {
            Err(APIError::HttpError(*status))
        },
        500..=599 => {
            Err(APIError::HttpError(*status))
        },
        _ => {
            let body = &response
                .bytes()
                .await?;
            
            match serde_json::from_slice::<D>(body) {
                Ok(body) => Ok(body),
                Err(parse_error) => {
                    // unexpected response
                    let html = String::from_utf8_lossy(body);
                    
                    if regex_is_match!(r#"<h1>Sorry!</h1>"#, &html) {
                        if let Some((_, message)) = regex_captures!("<h3>(.+)</h3>", &html) {
                            Err(APIError::ResponseError(message.into()))
                        } else {
                            Err(APIError::ResponseError("Unexpected error".into()))
                        }
                    } else if regex_is_match!(r#"<h1>Sign In</h1>"#, &html) && regex_is_match!(r#"g_steamID = false;"#, &html) {
                        Err(APIError::NotLoggedIn)
                    } else if let Some((_, message)) = regex_captures!(r#"<div id="error_msg">\s*([^<]+)\s*</div>"#, &html) {
                        Err(APIError::TradeError(message.into()))
                    } else {
                        Err(APIError::ParseError(parse_error))
                    }
                }
            }
        }
    }
}