//! HTTP Request/Response types

use std::collections::HashMap;

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
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
            ],
            body: body.to_string(),
        }
    }

    pub fn html(status: u16, body: &str) -> Self {
        Self {
            status,
            headers: vec![
                ("Content-Type".to_string(), "text/html; charset=utf-8".to_string()),
            ],
            body: body.to_string(),
        }
    }

    pub fn redirect(url: &str) -> Self {
        Self {
            status: 302,
            headers: vec![
                ("Location".to_string(), url.to_string()),
            ],
            body: String::new(),
        }
    }
}
