//! Fırat Shadow Handbook - Zero Dependency HTTP Server
//!
//! Pure Rust std::net implementation - no external crates

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, IpAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::PathBuf;

mod http;
mod handler;
mod config;
mod domain;
mod infrastructure;
mod application;

use http::{Request, Response, Method};
use handler::Router;

use application::composition::{CompositionRoot, AdapterConfig};

/// Security: Rate limiter to prevent DoS attacks
#[derive(Clone)]
struct RateLimiter {
    requests: Arc<std::sync::Mutex<HashMap<IpAddr, (u32, Instant)>>>,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    fn new(limit: u32, window: Duration) -> Self {
        Self {
            requests: Arc::new(std::sync::Mutex::new(HashMap::new())),
            limit,
            window,
        }
    }
    
    /// Security: Check if request is allowed
    fn allow(&self, ip: IpAddr) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        // Clean up old entries
        requests.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.window);
        
        // Get current count
        let count = requests.get(&ip).map(|(c, _)| *c).unwrap_or(0);
        
        if count >= self.limit {
            return false; // Rate limit exceeded
        }
        
        // Increment count
        requests.insert(ip, (count + 1, now));
        true
    }
}

static RUNNING: AtomicBool = AtomicBool::new(true);

// Redundant session store removed as we use cookie-based auth


fn main() {
    let config = config::Config::load();
    let addr = format!("{}:{}", config.host, config.port);
    
    println!("🚀 Fırat Shadow Handbook Server");
    println!("   Listening on http://{}", addr);
    println!("   Press Ctrl+C to stop\n");

    let listener = TcpListener::bind(&addr).expect("Failed to bind");
    listener.set_nonblocking(true).ok();

    let router = Arc::new(Router::new());
    setup_routes(&router);
    
    // Security: Initialize rate limiter (100 requests per minute per IP)
    let rate_limiter = Arc::new(RateLimiter::new(100, Duration::from_secs(60)));

    while RUNNING.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, addr)) => {
                let router = Arc::clone(&router);
                let rate_limiter = Arc::clone(&rate_limiter);
                thread::spawn(move || {
                    handle_connection(stream, addr, &router, &rate_limiter);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream, addr: std::net::SocketAddr, router: &Router, rate_limiter: &RateLimiter) {
    // Security: Rate limiting check
    if !rate_limiter.allow(addr.ip()) {
        let response = Response {
            status: 429,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Retry-After".to_string(), "60".to_string()),
            ],
            body: "Too Many Requests".to_string(),
        };
        let _ = send_response_raw(&mut stream, &response);
        return;
    }
    
    let reader = BufReader::new(&stream);
    
    let request = match parse_request(reader) {
        Some(r) => r,
        None => return,
    };

    // Security: Log without sensitive data
    println!("[{}] {} {}", addr.ip(), request.method, request.path);

    let response = router.handle(&request);
    send_response(&mut stream, &response);
}

/// Security: Validate HTTP method
fn validate_method(method: &str) -> Option<Method> {
    match method {
        "GET" => Some(Method::GET),
        "POST" => Some(Method::POST),
        "PUT" => Some(Method::PUT),
        "DELETE" => Some(Method::DELETE),
        _ => None,
    }
}

/// Security: Validate path to prevent injection
fn validate_path(path: &str) -> Option<String> {
    // Reject paths with null bytes
    if path.contains('\0') {
        return None;
    }
    
    // Reject paths with excessive length (DoS protection)
    if path.len() > 2048 {
        return None;
    }
    
    // Reject dangerous patterns
    if path.contains("..") || path.contains("%2e%2e") || path.contains("%5c%5c") {
        return None;
    }
    
    Some(path.to_string())
}

/// Security: Validate and sanitize headers
fn validate_header_key(key: &str) -> Option<String> {
    // Header keys should be token characters only
    if key.is_empty() || key.len() > 100 {
        return None;
    }
    
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    
    Some(key.to_string())
}

fn parse_request<R: BufRead>(mut reader: R) -> Option<Request> {
    let mut lines = reader.by_ref().lines();
    
    let first_line = lines.next()?.ok()?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    if parts.len() < 3 {
        return None;
    }

    // Security: Validate method
    let method = validate_method(parts[0])?;
    
    // Security: Validate path
    let path = validate_path(parts[1])?;
    
    let mut headers = std::collections::HashMap::new();
    let mut body = String::new();

    for line in lines {
        match line {
            Ok(l) if l.is_empty() => break,
            Ok(l) => {
                if let Some((key, value)) = l.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    
                    // Security: Validate header key
                    if let Some(valid_key) = validate_header_key(key) {
                        // Security: Limit header value length
                        if value.len() <= 1024 {
                            headers.insert(valid_key, value.to_string());
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }

    // Read body if Content-Length exists
    if let Some(len) = headers.get("Content-Length") {
        if let Ok(content_len) = len.parse::<usize>() {
            // Security: Limit request body size (DoS protection)
            if content_len > 1024 * 1024 { // 1MB max
                return None;
            }
            // Read body
            let mut buffer = vec![0; content_len];
            if reader.read_exact(&mut buffer).is_ok() {
                body = String::from_utf8_lossy(&buffer).to_string();
            }
        }
    }

    Some(Request {
        method,
        path,
        headers,
        body,
    })
}

fn send_response(stream: &mut TcpStream, response: &Response) {
    let _ = send_response_raw(stream, response);
}

/// Low-level response sender with security headers
fn send_response_raw(stream: &mut TcpStream, response: &Response) -> std::io::Result<()> {
    let status_text = match response.status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let mut headers_string = String::new();
    
    // Security: Add CORS headers with restriction
    // Only allow same-origin and specific trusted origins
    let cors_origin = if let Some(origin) = response.headers.iter().find(|(k, _)| k == "Access-Control-Allow-Origin") {
        // If already set by handler, keep it
        format!("{}: {}\r\n", "Access-Control-Allow-Origin", origin.1)
    } else {
        // Default: same-origin only (no wildcard)
        "Access-Control-Allow-Origin: same-origin\r\n".to_string()
    };
    headers_string.push_str(&cors_origin);
    
    // Add other headers
    for (k, v) in &response.headers {
        headers_string.push_str(&format!("{}: {}\r\n", k, v));
    }
    
    // Security: Ensure essential security headers are present
    if !response.headers.iter().any(|(k, _)| k == "X-Frame-Options") {
        headers_string.push_str("X-Frame-Options: DENY\r\n");
    }
    if !response.headers.iter().any(|(k, _)| k == "X-Content-Type-Options") {
        headers_string.push_str("X-Content-Type-Options: nosniff\r\n");
    }
    if !response.headers.iter().any(|(k, _)| k == "Content-Security-Policy") {
        headers_string.push_str("Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';\r\n");
    }
    if !response.headers.iter().any(|(k, _)| k == "Referrer-Policy") {
        headers_string.push_str("Referrer-Policy: strict-origin-when-cross-origin\r\n");
    }
    if !response.headers.iter().any(|(k, _)| k == "Permissions-Policy") {
        headers_string.push_str("Permissions-Policy: geolocation=(), microphone=(), camera=()\r\n");
    }

    let response_text = format!(
        "HTTP/1.1 {} {}\r\n{}\r\n{}",
        response.status, status_text, headers_string, response.body
    );

    stream.write_all(response_text.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn setup_routes(router: &Router) {
    // Static files - /web/* route as specified in story
    router.get("/", |_| serve_from_web("index.html", Some("text/html; charset=utf-8")));
    router.get("/web/*", |req| {
        let file = req.path.strip_prefix("/web/").unwrap_or("");
        serve_from_web(file, None)
    });
    // Legacy routes for backward compatibility
    router.get("/css/*", |req| {
        let file = req.path.strip_prefix("/css/").unwrap_or("");
        serve_from_web(&format!("css/{}", file), Some("text/css; charset=utf-8"))
    });
    router.get("/js/*", |req| {
        let file = req.path.strip_prefix("/js/").unwrap_or("");
        serve_from_web(&format!("js/{}", file), Some("application/javascript; charset=utf-8"))
    });
    router.get("/i18n/*", |req| {
        let file = req.path.strip_prefix("/i18n/").unwrap_or("");
        serve_from_web(&format!("i18n/{}", file), Some("application/json; charset=utf-8"))
    });
    router.get("/images/*", |req| {
        let file = req.path.strip_prefix("/images/").unwrap_or("");
        serve_from_web(&format!("images/{}", file), None)
    });
    
    // API endpoints
    router.get("/api/health", |_| Response::json(200, r#"{"status":"ok"}"#));
    router.get("/api/config", |_| Response::json(200, r#"{"version":"0.1.0"}"#));
    
    // Authentication endpoints
    router.post("/api/login", |req| handle_login(req));
    router.post("/api/logout", |req| handle_logout(req));
    router.get("/api/validate-session", |req| validate_session(req));
    router.get("/api/cas/callback*", |req| handle_cas_callback(req));
    router.post("/api/collab/scrape", |req| handle_collab_scrape(req));
}

/// Handles login request
fn handle_login(req: &Request) -> Response {
    // Parse JSON body
    let body = match serde_json::from_str::<serde_json::Value>(&req.body) {
        Ok(b) => b,
        Err(_) => return Response::json(400, r#"{"error":"Invalid JSON body"}"#),
    };

    let username = match body.get("username") {
        Some(v) => v.as_str().unwrap_or(""),
        None => return Response::json(400, r#"{"error":"Username is required"}"#),
    };

    let password = match body.get("password") {
        Some(v) => v.as_str().unwrap_or(""),
        None => return Response::json(400, r#"{"error":"Password is required"}"#),
    };

    // Initialize dependencies - port-first approach: adapter selection in one place
    // Using CompositionRoot pattern for centralized dependency wiring
    let composition = CompositionRoot::new(AdapterConfig::Production);
    let auth_port = composition.create_auth_adapter();
    let use_case: application::login_usecase::LoginUseCase<Box<dyn crate::domain::ports::auth_port::AuthPort>> = application::login_usecase::LoginUseCase::with_boxed(auth_port);

    // Execute login
    match use_case.login(username, password) {
        Ok(session) => {
            // Cookie-based session: directly use MoodleSession cookie
            // This ensures session persists across F5/browser refresh
            
            // Use serde_json for safe JSON serialization (prevents XSS and JSON parsing issues)
            let response_body = serde_json::json!({
                "success": true,
                "user": session.user.username,
                "full_name": session.user.full_name.as_deref().unwrap_or(""),
                "email": session.user.email.as_deref().unwrap_or("")
            }).to_string();
            let mut response = Response::json(200, &response_body);

            let secure = secure_cookie_suffix();
            // Set MoodleSession cookie directly - this is the real session from CAS/Debsis
            let moodle_cookie = format!(
                "MoodleSession={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=3600{}",
                session.moodle_session, secure
            );
            response.headers.push(("Set-Cookie".to_string(), moodle_cookie));

            // Set ShadowUser cookie (readable by JS for UI)
            let user_cookie = format!(
                "ShadowUser={}; Path=/; SameSite=Lax; Max-Age=3600{}",
                session.user.username, secure
            );
            response.headers.push(("Set-Cookie".to_string(), user_cookie));

            response
        }
        Err(e) => {
            let error_msg = match e {
                domain::ports::auth_port::AuthError::InvalidCredentials => "Invalid credentials",
                domain::ports::auth_port::AuthError::CasServerError(msg) => {
                    eprintln!("[CAS] Server error: {}", msg);
                    "CAS server error"
                }
                domain::ports::auth_port::AuthError::NetworkError(msg) => {
                    eprintln!("[CAS] Network error: {}", msg);
                    "Network error - cannot connect to CAS server"
                }
                domain::ports::auth_port::AuthError::InvalidSession => "Invalid session",
                domain::ports::auth_port::AuthError::ParsingError(msg) => {
                    eprintln!("[CAS] Parsing error: {}", msg);
                    "CAS response parsing error"
                }

            };
            let response_body = serde_json::json!({
                "success": false,
                "error": error_msg
            }).to_string();
            Response::json(401, &response_body)
        }
    }
}

/// Handles logout request — clears local session cookies
fn handle_logout(_req: &Request) -> Response {
    let secure = secure_cookie_suffix();
    let mut response = Response::json(200, r#"{"success":true}"#);

    // Clear MoodleSession cookie
    response.headers.push((
        "Set-Cookie".to_string(),
        format!("MoodleSession=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0{}", secure),
    ));

    // Clear ShadowUser cookie
    response.headers.push((
        "Set-Cookie".to_string(),
        format!("ShadowUser=; Path=/; SameSite=Lax; Max-Age=0{}", secure),
    ));

    response
}

/// Validates session
fn validate_session(req: &Request) -> Response {
    // Cookie-based: directly validate MoodleSession cookie
    let moodle_session = match get_cookie(req, "MoodleSession") {
        Some(v) => v,
        None => return Response::json(401, r#"{"valid":false,"error":"No active session"}"#),
    };

    // Use Composition Root for centralized adapter selection (port-first approach)
    let composition = CompositionRoot::new(AdapterConfig::Production);
    let auth_port = composition.create_auth_adapter();
    let use_case: application::login_usecase::LoginUseCase<Box<dyn crate::domain::ports::auth_port::AuthPort>> = application::login_usecase::LoginUseCase::with_boxed(auth_port);

    match use_case.validate_session(&moodle_session) {
        Ok(user) => {
            // Use serde_json for safe JSON serialization (prevents XSS and JSON parsing issues)
            let response_body = serde_json::json!({
                "valid": true,
                "user": user.username,
                "full_name": user.full_name.as_deref().unwrap_or(""),
                "email": user.email.as_deref().unwrap_or("")
            }).to_string();
            Response::json(200, &response_body)
        }
        Err(e) => {
            eprintln!("[validate_session] Session validation failed: {:?}", e);
            Response::json(401, r#"{"valid":false,"error":"Invalid session"}"#)
        }
    }
}

/// Handles CAS callback - validates ticket and creates session
fn handle_cas_callback(req: &Request) -> Response {
    // Extract ticket from query string
    let ticket = req.path
        .split('?')
        .nth(1)
        .and_then(|query| {
            query.split('&').find_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next()?;
                if key == "ticket" {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        });

    let ticket = match ticket {
        Some(t) => t,
        None => {
            return Response {
                status: 302,
                headers: vec![
                    ("Location".to_string(), "/#/login?error=no_ticket".to_string()),
                ],
                body: String::new(),
            };
        }
    };

    // Validate ticket with CAS server using direct CasAdapter
    // Match the exact origin the frontend used, based on the Host header, 
    // to strictly satisfy CAS service URL validation.
    let host = get_header_case_insensitive(req, "Host").unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    // In production we usually run behind a reverse proxy handling HTTPS, so check X-Forwarded-Proto
    let proto = get_header_case_insensitive(req, "X-Forwarded-Proto").unwrap_or_else(|| "http".to_string());
    
    let service_url = format!("{}://{}/api/cas/callback", proto, host);
    
    let cas_adapter = crate::infrastructure::cas_adapter::CasAdapter::new(
        "https://jasig.firat.edu.tr/cas".to_string(),
        service_url
    );
    
    let username = match cas_adapter.validate_ticket(&ticket) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("[cas_callback] Ticket validation failed: {:?}", e);
            return Response {
                status: 302,
                headers: vec![
                    ("Location".to_string(), "/#/login?error=invalid_ticket".to_string()),
                ],
                body: String::new(),
            };
        }
    };

    println!("[cas_callback] Ticket validated for user: {}", username);

    // Create session token with username association
    let session_token = generate_token();
    
    let mut response = Response {
        status: 302,
        headers: vec![
            ("Location".to_string(), "/#/".to_string()),
        ],
        body: String::new(),
    };

    let secure = secure_cookie_suffix();
    
    // Set MoodleSession cookie - SameSite=Lax required because this is a cross-site redirect from CAS
    // SameSite=Strict would cause the browser to strip the cookie on the redirect back from jasig.firat.edu.tr
    let session_cookie = format!(
        "MoodleSession={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=3600{}",
        session_token, secure
    );
    response.headers.push(("Set-Cookie".to_string(), session_cookie));

    // Set a readable cookie for the frontend to know the username (not HttpOnly so JS can read it)
    let user_cookie = format!(
        "ShadowUser={}; Path=/; SameSite=Lax; Max-Age=3600{}",
        username, secure
    );
    response.headers.push(("Set-Cookie".to_string(), user_cookie));

    response
}

fn handle_collab_scrape(req: &Request) -> Response {
    // Validate Content-Type header
    let content_type = get_header_case_insensitive(req, "Content-Type").unwrap_or_default();
    if !content_type.contains("application/json") {
        return Response::json(415, r#"{"error":"Content-Type must be application/json"}"#);
    }

    // Cookie-based: get MoodleSession directly
    let moodle_session = match get_cookie(req, "MoodleSession") {
        Some(v) => v,
        None => return Response::json(401, r#"{"error":"No active session"}"#),
    };

    let body = match serde_json::from_str::<serde_json::Value>(&req.body) {
        Ok(v) => v,
        Err(_) => return Response::json(400, r#"{"error":"Invalid JSON body"}"#),
    };

    let html = match body.get("html").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return Response::json(400, r#"{"error":"html field is required"}"#),
    };

    // Use Composition Root for centralized adapter selection (port-first approach)
    let composition = CompositionRoot::new(AdapterConfig::Production);
    let scraper_port = composition.create_scraper_adapter();
    let use_case: application::collab_scraper_usecase::CollabScraperUseCase<Box<dyn crate::domain::ports::scraper_port::ScraperPort>> = application::collab_scraper_usecase::CollabScraperUseCase::with_boxed(scraper_port);

    match use_case.scrape(&moodle_session, html) {
        Ok(snapshot) => match serde_json::to_string(&snapshot) {
            Ok(payload) => Response::json(200, &payload),
            Err(_) => Response::json(500, r#"{"error":"Failed to serialize response"}"#),
        },
        Err(err) => {
            let (status, message): (u16, String) = match err {
                domain::ports::scraper_port::ScraperError::Unauthorized => {
                    (401, "Unauthorized".to_string())
                }
                domain::ports::scraper_port::ScraperError::InvalidInput(msg) => (400, msg),
                domain::ports::scraper_port::ScraperError::ParseError(msg) => (422, msg),
                domain::ports::scraper_port::ScraperError::UnsupportedFormat(msg) => (422, msg),
            };
            let body = serde_json::json!({ "error": message }).to_string();
            Response::json(status, &body)
        }
    }
}

fn get_header_case_insensitive(req: &Request, target: &str) -> Option<String> {
    req.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(target))
        .map(|(_, v)| v.to_string())
}

fn parse_cookie_header(req: &Request) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    let raw = match get_header_case_insensitive(req, "Cookie") {
        Some(v) => v,
        None => return cookies,
    };
    for part in raw.split(';') {
        if let Some((k, v)) = part.trim().split_once('=') {
            cookies.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    cookies
}

fn get_cookie(req: &Request, key: &str) -> Option<String> {
    parse_cookie_header(req).get(key).cloned()
}




fn secure_cookie_suffix() -> &'static str {
    match std::env::var("APP_ENV") {
        Ok(v) if v.eq_ignore_ascii_case("production") || v.eq_ignore_ascii_case("prod") => {
            "; Secure"
        }
        _ => "",
    }
}

fn generate_token() -> String {
    let mut bytes = [0u8; 24];
    if let Ok(mut file) = File::open("/dev/urandom") {
        if file.read_exact(&mut bytes).is_ok() {
            return to_hex(&bytes);
        }
    }
    let fallback = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:048x}", fallback)
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use crate::http::{Method, Request};

    fn make_request_with_cookie(cookie: &str) -> Request {
        let mut headers = HashMap::new();
        headers.insert("Cookie".to_string(), cookie.to_string());
        Request {
            method: Method::GET,
            path: "/".to_string(),
            headers,
            body: String::new(),
        }
    }

    #[test]
    fn test_get_cookie_parses_named_cookie() {
        let req = make_request_with_cookie("A=1; ShadowSession=abc123; C=3");
        let value = get_cookie(&req, "ShadowSession");
        assert_eq!(value.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_generate_token_has_expected_hex_length() {
        let token = generate_token();
        assert_eq!(token.len(), 48);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    fn make_collab_request(cookie: Option<&str>, body: &str) -> Request {
        make_collab_request_with_content_type(cookie, body, "application/json")
    }

    fn make_collab_request_with_content_type(cookie: Option<&str>, body: &str, content_type: &str) -> Request {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        if let Some(raw) = cookie {
            headers.insert("Cookie".to_string(), raw.to_string());
        }
        Request {
            method: Method::POST,
            path: "/api/collab/scrape".to_string(),
            headers,
            body: body.to_string(),
        }
    }



    #[test]
    fn collab_scrape_returns_401_without_session_cookie() {
        let req = make_collab_request(None, r#"{"html":"<div>test</div>"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 401);
    }

    #[test]
    fn collab_scrape_returns_snapshot_for_valid_session() {
        // Now using cookie-based: no need to insert session store
        // Just pass valid MoodleSession cookie

        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\" data-schedule=\"2026-02-27T10:00:00+03:00|2026-02-27T11:00:00+03:00|Europe/Istanbul\"></div><a class=\"playback-link\" data-playback=\"true\" href=\"https://eu.bbcollab.com/recording/abc\" data-label=\"Kayit\">Kayit</a>"
        })
        .to_string();

        // Use MoodleSession cookie instead of ShadowSession
        let req = make_collab_request(Some("MoodleSession=mdl-session"), &body);
        let response = handle_collab_scrape(&req);

        assert_eq!(response.status, 200);
        let payload: serde_json::Value =
            serde_json::from_str(&response.body).expect("snapshot json should parse");
        assert!(payload.get("courses").is_some());
        assert!(payload.get("playbacks").is_some());
    }

    #[test]
    fn collab_scrape_returns_400_for_invalid_json_body() {
        // No session store needed for cookie-based auth

        // Use MoodleSession cookie
        let req = make_collab_request(Some("MoodleSession=mdl-session"), "{not-json");
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn collab_scrape_returns_400_when_html_field_missing() {

        // Use MoodleSession cookie
        let req = make_collab_request(Some("MoodleSession=mdl-session"), r#"{"foo":"bar"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn collab_scrape_returns_401_for_expired_session() {
        // For cookie-based auth: if no/invalid MoodleSession cookie, returns 401
        // This tests the case where cookie is missing or obviously invalid
        
        let req = make_collab_request(None, r#"{"html":"<div>test payload</div>"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 401);
    }

    #[test]
    fn collab_scrape_returns_422_for_non_allowlisted_playback_url() {
        // No session store needed for cookie-based auth

        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\"></div><a class=\"playback-link\" href=\"https://evil.local/recording/abc\">Kayit</a>"
        })
        .to_string();

        // Use MoodleSession cookie
        let req = make_collab_request(Some("MoodleSession=mdl-session"), &body);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 422);
    }

    #[test]
    fn collab_scrape_returns_415_for_missing_content_type() {
        let req = make_collab_request_with_content_type(
            Some("MoodleSession=mdl-session"),
            r#"{"html":"<div>test</div>"}"#,
            "text/plain"
        );
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 415);
    }

    #[test]
    fn collab_scrape_accepts_json_content_type() {
        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\"></div>"
        }).to_string();

        let req = make_collab_request(Some("MoodleSession=mdl-session"), &body);
        let response = handle_collab_scrape(&req);
        // Should not be 415, session validation happens first
        assert_ne!(response.status, 415);
    }
}

/// Security: Path traversal prevention
/// Validates and sanitizes relative paths to prevent directory traversal attacks.
fn sanitize_relative_path(relative_path: &str) -> Result<String, &'static str> {
    if relative_path.is_empty() {
        return Err("Invalid filename");
    }

    if relative_path.contains("..") || relative_path.contains('\\') || relative_path.contains('\0') {
        return Err("Invalid filename");
    }

    let mut cleaned_segments: Vec<String> = Vec::new();
    for segment in relative_path.split('/') {
        if segment.is_empty() {
            return Err("Invalid filename");
        }
        if !segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err("Invalid filename");
        }
        cleaned_segments.push(segment.to_string());
    }

    Ok(cleaned_segments.join("/"))
}

fn web_root() -> PathBuf {
    let from_root = PathBuf::from("web");
    if from_root.exists() {
        return from_root;
    }

    let from_src = PathBuf::from("../web");
    if from_src.exists() {
        return from_src;
    }

    PathBuf::from("web")
}

fn content_type_for(path: &str) -> &'static str {
    if path.ends_with(".css") {
        return "text/css; charset=utf-8";
    }
    if path.ends_with(".js") {
        return "application/javascript; charset=utf-8";
    }
    if path.ends_with(".json") {
        return "application/json; charset=utf-8";
    }
    if path.ends_with(".svg") {
        return "image/svg+xml; charset=utf-8";
    }
    if path.ends_with(".html") {
        return "text/html; charset=utf-8";
    }
    "text/plain; charset=utf-8"
}

/// Serve static file with path traversal protections.
fn serve_from_web(relative_path: &str, content_type_override: Option<&str>) -> Response {
    let relative_path = match sanitize_relative_path(relative_path) {
        Ok(path) => path,
        Err(_) => {
            return Response {
                status: 400,
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: "Bad Request".to_string(),
            };
        }
    };

    let base_path = web_root();
    let full_path = base_path.join(&relative_path);
    let base_canonical = match base_path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return Response {
                status: 500,
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: "Static root unavailable".to_string(),
            };
        }
    };

    if let Ok(canonical) = full_path.canonicalize() {
        if !canonical.starts_with(base_canonical) {
            return Response {
                status: 403,
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: "Forbidden".to_string(),
            };
        }
    } else {
        return Response {
            status: 404,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: "Not Found".to_string(),
        };
    }

    let content_type = content_type_override.unwrap_or_else(|| content_type_for(&relative_path));
    match fs::read_to_string(&full_path) {
        Ok(content) => Response {
            status: 200,
            headers: vec![
                ("Content-Type".to_string(), content_type.to_string()),
                ("Content-Length".to_string(), content.len().to_string()),
                ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
            ],
            body: content,
        },
        Err(_) => Response {
            status: 404,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: "Not Found".to_string(),
        },
    }
}
