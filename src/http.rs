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
        let mut response = Self {
            status,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: body.to_string(),
        };
        response.add_security_headers();
        response
    }

    pub fn html(status: u16, body: &str) -> Self {
        let mut response = Self {
            status,
            headers: vec![
                ("Content-Type".to_string(), "text/html; charset=utf-8".to_string()),
            ],
            body: body.to_string(),
        };
        response.add_security_headers();
        response
    }

    pub fn redirect(url: &str) -> Self {
        let mut response = Self {
            status: 302,
            headers: vec![
                ("Location".to_string(), url.to_string()),
            ],
            body: String::new(),
        };
        response.add_security_headers();
        response
    }

    /// Security: Add standard security headers to all responses
    fn add_security_headers(&mut self) {
        // Prevent clickjacking
        if !self.headers.iter().any(|(k, _)| k == "X-Frame-Options") {
            self.headers.push(("X-Frame-Options".to_string(), "DENY".to_string()));
        }
        
        // Prevent MIME type sniffing
        if !self.headers.iter().any(|(k, _)| k == "X-Content-Type-Options") {
            self.headers.push(("X-Content-Type-Options".to_string(), "nosniff".to_string()));
        }
        
        // Content Security Policy (basic)
        if !self.headers.iter().any(|(k, _)| k == "Content-Security-Policy") {
            let csp = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';".to_string();
            self.headers.push(("Content-Security-Policy".to_string(), csp));
        }
        
        // Referrer Policy
        if !self.headers.iter().any(|(k, _)| k == "Referrer-Policy") {
            self.headers.push(("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string()));
        }
        
        // Permissions Policy
        if !self.headers.iter().any(|(k, _)| k == "Permissions-Policy") {
            self.headers.push(("Permissions-Policy".to_string(), "geolocation=(), microphone=(), camera=()".to_string()));
        }
    }
}
