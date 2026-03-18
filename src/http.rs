//! HTTP Request/Response types

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::DELETE => write!(f, "DELETE"),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    /// Cached parsed cookies - computed once on first access
    cookies: OnceLock<HashMap<String, String>>,
}

impl Request {
    /// Create a new request (cookies will be parsed lazily)
    pub fn new(
        method: Method,
        path: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            method,
            path,
            headers,
            body,
            cookies: OnceLock::new(),
        }
    }

    /// Get a cookie value by name (cached)
    pub fn get_cookie(&self, name: &str) -> Option<&str> {
        let cookies = self.cookies.get_or_init(|| {
            let mut map = HashMap::new();
            if let Some(raw) = self
                .headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("Cookie"))
                .map(|(_, v)| v.as_str())
            {
                for part in raw.split(';') {
                    if let Some((k, v)) = part.trim().split_once('=') {
                        map.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
            }
            map
        });
        cookies.get(name).map(|s| s.as_str())
    }
}

pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Response {
    pub fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: body.to_string(),
        }
    }

    pub fn html(status: u16, body: &str) -> Self {
        Self {
            status,
            headers: vec![(
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            )],
            body: body.to_string(),
        }
    }

    /// Creates a redirect response (302 Found)
    /// Used for login/logout flows and post-redirect-get pattern
    #[allow(dead_code)]
    pub fn redirect(url: &str) -> Self {
        Self {
            status: 302,
            headers: vec![("Location".to_string(), url.to_string())],
            body: String::new(),
        }
    }
}
