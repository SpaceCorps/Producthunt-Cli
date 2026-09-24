//! HTTP client for the Apify API driving the Product Hunt scraper actor.
//!
//! One blocking agent per process: a CLI makes a single search or scrape call, so an async runtime
//! would cost more in startup than it could save.

use std::time::Duration;

use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};

const DEFAULT_BASE: &str = "https://api.apify.com/v2/";
const MAX_BODY: u64 = 512 * 1024 * 1024;
const ACTOR_ENDPOINT: &str = "acts/cloud9_ai~producthunt-scraper/run-sync-get-dataset-items";

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    token: String,
}

enum Method {
    Get,
    Post,
}

impl Client {
    pub fn new(token: &str) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(300)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("producthunt-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut base = std::env::var("PRODUCTHUNT_API_URL")
            .or_else(|_| std::env::var("APIFY_API_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, token: token.trim().to_string() }
    }

    #[allow(dead_code)]
    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn me(&self) -> Result<Value> {
        self.get("users/me")
    }

    pub fn search(&self, input: &Value) -> Result<Value> {
        let endpoint = format!("{ACTOR_ENDPOINT}?token={}", self.token);
        self.post(&endpoint, input)
    }

    pub fn scrape(&self, input: &Value) -> Result<Value> {
        let endpoint = format!("{ACTOR_ENDPOINT}?token={}", self.token);
        self.post(&endpoint, input)
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Post, path, Some(body))
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let clean_path = path.strip_prefix('/').unwrap_or(path);
        let url = format!("{}{}", self.base, clean_path);

        let auth_header = format!("Bearer {}", self.token);
        macro_rules! headers {
            ($req:expr) => {
                $req.header("Authorization", &auth_header).header("Accept", "application/json")
            };
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Post, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send(&json[..])
            }
            (Method::Post, None) => headers!(self.agent.post(&url)).send_empty(),
        };

        let response = result.map_err(transport_error)?;
        read(response)
    }
}

fn read(mut response: Response<ureq::Body>) -> Result<Value> {
    let status = response.status().as_u16();
    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Object(Default::default()));
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the Apify API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    let api_message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| {
            v.get("error")
                .and_then(|e| e.get("message").or_else(|| e.get("error")))
                .or_else(|| v.get("message"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();

    let e = match status {
        401 => Error::new(
            ErrorCode::AuthRequired,
            if api_message.is_empty() { "The Apify API token was rejected or is invalid.".into() } else { api_message },
        )
        .fix("Replace it: producthunt login <name> --force or set APIFY_TOKEN=<token>"),
        403 => Error::new(
            ErrorCode::AuthRequired,
            if api_message.is_empty() {
                "Access forbidden. Your Apify account does not have permission for this resource.".into()
            } else {
                api_message
            },
        )
        .fix("Check your Apify account and token permissions: https://console.apify.com/account/integrations"),
        404 => Error::new(
            ErrorCode::NotFound,
            if api_message.is_empty() { "The requested resource was not found.".into() } else { api_message },
        ),
        429 => Error::new(ErrorCode::RateLimited, "Rate limited by the Apify API.").fix("Back off before retrying."),
        400 | 422 => Error::new(
            ErrorCode::InvalidInput,
            if api_message.is_empty() { "The Apify API refused the request.".into() } else { api_message },
        ),
        s if s >= 500 => Error::new(ErrorCode::Network, "The Apify API returned a server error.")
            .fix("Retry; if it persists Apify may be experiencing an outage."),
        _ => Error::new(ErrorCode::Error, "The request failed."),
    };
    e.detail(detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(403, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(400, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(422, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(500, "").code, ErrorCode::Network);
        assert_eq!(status_error(503, "").code, ErrorCode::Network);
    }

    #[test]
    fn parses_apify_error_message() {
        let err_json = r#"{"error":{"message":"Token is invalid!","type":"INVALID_TOKEN"}}"#;
        let err = status_error(401, err_json);
        assert_eq!(err.message, "Token is invalid!");
        assert_eq!(err.code, ErrorCode::AuthRequired);
    }
}
