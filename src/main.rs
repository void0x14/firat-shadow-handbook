//! Fırat Shadow Handbook - Zero Dependency HTTP Server
//!
//! Pure Rust std::net implementation - no external crates

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, IpAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
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
use domain::user::User;
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
static SESSION_STORE: OnceLock<Mutex<HashMap<String, AppSession>>> = OnceLock::new();

#[derive(Clone)]
struct AppSession {
    moodle_session: String,
    user: User,
    csrf_token: String,
    expires_at: Instant,
}

fn session_store() -> &'static Mutex<HashMap<String, AppSession>> {
    SESSION_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

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
    // Static files
    router.get("/", |_| serve_from_web("index.html", Some("text/html; charset=utf-8")));
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
            if let Some(old_shadow) = get_cookie(req, "ShadowSession") {
                session_store().lock().unwrap().remove(&old_shadow);
            }

            let shadow_session = generate_token();
            let csrf_token = generate_token();
            let expires_at = Instant::now() + Duration::from_secs(3600);

            session_store().lock().unwrap().insert(
                shadow_session.clone(),
                AppSession {
                    moodle_session: session.moodle_session.clone(),
                    user: session.user.clone(),
                    csrf_token: csrf_token.clone(),
                    expires_at,
                },
            );

            let mut response = Response::json(200, &format!(
                r#"{{"success":true,"user":"{}","full_name":"{}","email":"{}"}}"#,
                session.user.username,
                session.user.full_name.as_ref().unwrap_or(&"".to_string()),
                session.user.email.as_ref().unwrap_or(&"".to_string())
            ));

            let secure = secure_cookie_suffix();
            let shadow_cookie = format!(
                "ShadowSession={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=3600{}",
                shadow_session, secure
            );
            let moodle_cookie = format!(
                "MoodleSession={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=3600{}",
                session.moodle_session, secure
            );
            let csrf_cookie = format!(
                "CSRF-Token={}; Path=/; SameSite=Strict; Max-Age=3600{}",
                csrf_token, secure
            );
            response.headers.push(("Set-Cookie".to_string(), shadow_cookie));
            response.headers.push(("Set-Cookie".to_string(), moodle_cookie));
            response.headers.push(("Set-Cookie".to_string(), csrf_cookie));

            response
        }
        Err(e) => {
            let error_msg = match e {
                domain::ports::auth_port::AuthError::InvalidCredentials => "Invalid credentials",
                domain::ports::auth_port::AuthError::CasServerError(_) => "CAS server error",
                domain::ports::auth_port::AuthError::NetworkError(_) => "Network error",
                domain::ports::auth_port::AuthError::InvalidSession => "Invalid session",
                domain::ports::auth_port::AuthError::ParsingError(_) => "Parsing error",
                domain::ports::auth_port::AuthError::Unknown(_) => "Unknown error",
            };
            Response::json(401, &format!(r#"{{"success":false,"error":"{}"}}"#, error_msg))
        }
    }
}

/// Handles logout request
fn handle_logout(req: &Request) -> Response {
    let shadow_session = match get_cookie(req, "ShadowSession") {
        Some(c) => c,
        None => return Response::json(401, r#"{"success":false,"error":"No active session"}"#),
    };
    let csrf_header = get_header_case_insensitive(req, "X-CSRF-Token");
    let csrf_cookie = get_cookie(req, "CSRF-Token");

    let app_session = {
        let store = session_store().lock().unwrap();
        match store.get(&shadow_session) {
            Some(session) => session.clone(),
            None => {
                return Response::json(401, r#"{"success":false,"error":"Session not found"}"#);
            }
        }
    };

    if !validate_csrf(csrf_header.as_deref(), csrf_cookie.as_deref(), &app_session.csrf_token) {
        return Response::json(403, r#"{"success":false,"error":"Invalid CSRF token"}"#);
    }

    // Use Composition Root for centralized adapter selection (port-first approach)
    let composition = CompositionRoot::new(AdapterConfig::Production);
    let auth_port = composition.create_auth_adapter();
    let use_case: application::login_usecase::LoginUseCase<Box<dyn crate::domain::ports::auth_port::AuthPort>> = application::login_usecase::LoginUseCase::with_boxed(auth_port);

    match use_case.logout(&app_session.moodle_session) {
        Ok(_) => {
            session_store().lock().unwrap().remove(&shadow_session);
            let mut response = Response::json(200, r#"{"success":true}"#);
            let secure = secure_cookie_suffix();
            response.headers.push((
                "Set-Cookie".to_string(),
                format!(
                    "ShadowSession=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0{}",
                    secure
                ),
            ));
            response.headers.push((
                "Set-Cookie".to_string(),
                format!(
                    "MoodleSession=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0{}",
                    secure
                ),
            ));
            response.headers.push((
                "Set-Cookie".to_string(),
                format!("CSRF-Token=; Path=/; SameSite=Strict; Max-Age=0{}", secure),
            ));
            response
        }
        Err(_) => Response::json(401, r#"{"success":false,"error":"Invalid session"}"#),
    }
}

/// Validates session
fn validate_session(req: &Request) -> Response {
    let shadow_session = match get_cookie(req, "ShadowSession") {
        Some(v) => v,
        None => return Response::json(401, r#"{"valid":false,"error":"No active session"}"#),
    };

    let app_session = {
        let mut store = session_store().lock().unwrap();
        let session = match store.get(&shadow_session) {
            Some(s) => s.clone(),
            None => return Response::json(401, r#"{"valid":false,"error":"Invalid session"}"#),
        };

        if Instant::now() > session.expires_at {
            store.remove(&shadow_session);
            return Response::json(401, r#"{"valid":false,"error":"Session expired"}"#);
        }
        session
    };

    // Use Composition Root for centralized adapter selection (port-first approach)
    let composition = CompositionRoot::new(AdapterConfig::Production);
    let auth_port = composition.create_auth_adapter();
    let use_case: application::login_usecase::LoginUseCase<Box<dyn crate::domain::ports::auth_port::AuthPort>> = application::login_usecase::LoginUseCase::with_boxed(auth_port);

    match use_case.validate_session(&app_session.moodle_session) {
        Ok(_user) => Response::json(200, &format!(
            r#"{{"valid":true,"user":"{}","full_name":"{}","email":"{}"}}"#,
            app_session.user.username,
            app_session.user.full_name.as_ref().unwrap_or(&"".to_string()),
            app_session.user.email.as_ref().unwrap_or(&"".to_string())
        )),
        Err(_) => Response::json(401, r#"{"valid":false,"error":"Invalid session"}"#),
    }
}

fn handle_collab_scrape(req: &Request) -> Response {
    // Validate Content-Type header
    let content_type = get_header_case_insensitive(req, "Content-Type").unwrap_or_default();
    if !content_type.contains("application/json") {
        return Response::json(415, r#"{"error":"Content-Type must be application/json"}"#);
    }

    let shadow_session = match get_cookie(req, "ShadowSession") {
        Some(v) => v,
        None => return Response::json(401, r#"{"error":"No active session"}"#),
    };

    let app_session = {
        let mut store = session_store().lock().unwrap();
        let session = match store.get(&shadow_session) {
            Some(s) => s.clone(),
            None => return Response::json(401, r#"{"error":"Invalid session"}"#),
        };

        if Instant::now() > session.expires_at {
            store.remove(&shadow_session);
            return Response::json(401, r#"{"error":"Session expired"}"#);
        }
        session
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

    match use_case.scrape(&app_session.moodle_session, html) {
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
                domain::ports::scraper_port::ScraperError::Unknown(msg) => (422, msg),
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

fn validate_csrf(header: Option<&str>, cookie: Option<&str>, expected: &str) -> bool {
    match (header, cookie) {
        (Some(h), Some(c)) => h == expected && c == expected,
        _ => false,
    }
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
    fn test_validate_csrf_requires_header_cookie_and_expected_match() {
        assert!(validate_csrf(Some("token"), Some("token"), "token"));
        assert!(!validate_csrf(Some("token"), Some("other"), "token"));
        assert!(!validate_csrf(None, Some("token"), "token"));
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

    fn reset_session_store() {
        session_store().lock().unwrap().clear();
    }

    #[test]
    fn collab_scrape_returns_401_without_session_cookie() {
        reset_session_store();
        let req = make_collab_request(None, r#"{"html":"<div>test</div>"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 401);
    }

    #[test]
    fn collab_scrape_returns_snapshot_for_valid_session() {
        reset_session_store();

        session_store().lock().unwrap().insert(
            "shadow-1".to_string(),
            AppSession {
                moodle_session: "mdl-session".to_string(),
                user: User::new("tester".to_string()),
                csrf_token: "csrf".to_string(),
                expires_at: Instant::now() + Duration::from_secs(600),
            },
        );

        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\" data-schedule=\"2026-02-27T10:00:00+03:00|2026-02-27T11:00:00+03:00|Europe/Istanbul\"></div><a class=\"playback-link\" data-playback=\"true\" href=\"https://eu.bbcollab.com/recording/abc\" data-label=\"Kayit\">Kayit</a>"
        })
        .to_string();

        let req = make_collab_request(Some("ShadowSession=shadow-1"), &body);
        let response = handle_collab_scrape(&req);

        assert_eq!(response.status, 200);
        let payload: serde_json::Value =
            serde_json::from_str(&response.body).expect("snapshot json should parse");
        assert!(payload.get("courses").is_some());
        assert!(payload.get("playbacks").is_some());
    }

    #[test]
    fn collab_scrape_returns_400_for_invalid_json_body() {
        reset_session_store();
        session_store().lock().unwrap().insert(
            "shadow-1".to_string(),
            AppSession {
                moodle_session: "mdl-session".to_string(),
                user: User::new("tester".to_string()),
                csrf_token: "csrf".to_string(),
                expires_at: Instant::now() + Duration::from_secs(600),
            },
        );

        let req = make_collab_request(Some("ShadowSession=shadow-1"), "{not-json");
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn collab_scrape_returns_400_when_html_field_missing() {
        reset_session_store();
        session_store().lock().unwrap().insert(
            "shadow-1".to_string(),
            AppSession {
                moodle_session: "mdl-session".to_string(),
                user: User::new("tester".to_string()),
                csrf_token: "csrf".to_string(),
                expires_at: Instant::now() + Duration::from_secs(600),
            },
        );

        let req = make_collab_request(Some("ShadowSession=shadow-1"), r#"{"foo":"bar"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn collab_scrape_returns_401_for_expired_session() {
        reset_session_store();
        session_store().lock().unwrap().insert(
            "shadow-1".to_string(),
            AppSession {
                moodle_session: "mdl-session".to_string(),
                user: User::new("tester".to_string()),
                csrf_token: "csrf".to_string(),
                expires_at: Instant::now() - Duration::from_secs(1),
            },
        );

        let req = make_collab_request(Some("ShadowSession=shadow-1"), r#"{"html":"<div>test payload</div>"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 401);
    }

    #[test]
    fn collab_scrape_returns_422_for_non_allowlisted_playback_url() {
        reset_session_store();
        session_store().lock().unwrap().insert(
            "shadow-1".to_string(),
            AppSession {
                moodle_session: "mdl-session".to_string(),
                user: User::new("tester".to_string()),
                csrf_token: "csrf".to_string(),
                expires_at: Instant::now() + Duration::from_secs(600),
            },
        );

        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\"></div><a class=\"playback-link\" href=\"https://evil.local/recording/abc\">Kayit</a>"
        })
        .to_string();

        let req = make_collab_request(Some("ShadowSession=shadow-1"), &body);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 422);
    }

    #[test]
    fn collab_scrape_returns_415_for_missing_content_type() {
        reset_session_store();
        session_store().lock().unwrap().insert(
            "shadow-1".to_string(),
            AppSession {
                moodle_session: "mdl-session".to_string(),
                user: User::new("tester".to_string()),
                csrf_token: "csrf".to_string(),
                expires_at: Instant::now() + Duration::from_secs(600),
            },
        );

        let req = make_collab_request_with_content_type(
            Some("ShadowSession=shadow-1"),
            r#"{"html":"<div>test</div>"}"#,
            "text/plain"
        );
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 415);
    }

    #[test]
    fn collab_scrape_accepts_json_content_type() {
        reset_session_store();
        session_store().lock().unwrap().insert(
            "shadow-1".to_string(),
            AppSession {
                moodle_session: "mdl-session".to_string(),
                user: User::new("tester".to_string()),
                csrf_token: "csrf".to_string(),
                expires_at: Instant::now() + Duration::from_secs(600),
            },
        );

        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\"></div>"
        }).to_string();

        let req = make_collab_request(Some("ShadowSession=shadow-1"), &body);
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
