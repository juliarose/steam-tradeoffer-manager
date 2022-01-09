use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest::{header, cookie::CookieStore};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use crate::APIError;
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

pub async fn parses_response<D>(response: reqwest::Response) -> Result<D, APIError>
where
    D: DeserializeOwned
{
    #[derive(Deserialize, Debug)]
    struct ErrorResponse {
        message: String,
    }
    
    let body = &response
        .bytes()
        .await?;
    
    match serde_json::from_slice::<D>(body) {
        Ok(body) => Ok(body),
        Err(parse_error) => {
            // unexpected response
            if let Ok(error_body) = serde_json::from_slice::<ErrorResponse>(body) { 
                Err(APIError::ResponseError(error_body.message.into()))
            } else {
                Err(parse_error.into())
            }
        }
    }
}