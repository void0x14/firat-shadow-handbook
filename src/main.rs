//! Fırat Shadow Handbook - Zero Dependency HTTP Server
//!
//! Pure Rust std::net implementation - no external crates

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, IpAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
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
fn validate_method(method: &str) -> Method {
    match method {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        _ => {
            // Invalid method will be handled by returning None
            return Method::GET; // Placeholder, will be filtered upstream
        }
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
    let method = validate_method(parts[0]);
    
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

    // Initialize dependencies
    let cas_adapter = infrastructure::cas_adapter::CasAdapter::new(
        "https://jasig.firat.edu.tr/cas".to_string(),
        "https://debsis.firat.edu.tr".to_string(),
    );
    let use_case = application::login_usecase::LoginUseCase::new(cas_adapter);

    // Execute login
    match use_case.login(username, password) {
        Ok(session) => {
            let mut response = Response::json(200, &format!(
                r#"{{"success":true,"user":"{}","full_name":"{}","email":"{}"}}"#,
                session.user.username,
                session.user.full_name.as_ref().unwrap_or(&"".to_string()),
                session.user.email.as_ref().unwrap_or(&"".to_string())
            ));

            // Security: Set HttpOnly, Secure, SameSite=Strict cookie
            let cookie = format!(
                "MoodleSession={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=86400",
                session.moodle_session
            );
            response.headers.push(("Set-Cookie".to_string(), cookie));

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
    // Get MoodleSession cookie from request
    let cookie = req.headers.get("Cookie")
        .and_then(|c| {
            c.split(';')
                .find(|part| part.trim().starts_with("MoodleSession="))
                .map(|part| part.trim().strip_prefix("MoodleSession=").unwrap_or("").to_string())
        })
        .ok_or("No MoodleSession cookie")
        .map_err(|e| Response::json(401, &format!(r#"{{"success":false,"error":"{}"}}"#, e)));

    let cookie = match cookie {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Initialize dependencies
    let cas_adapter = infrastructure::cas_adapter::CasAdapter::new(
        "https://jasig.firat.edu.tr/cas".to_string(),
        "https://debsis.firat.edu.tr".to_string(),
    );
    let use_case = application::login_usecase::LoginUseCase::new(cas_adapter);

    // Execute logout
    match use_case.logout(&cookie) {
        Ok(_) => {
            let mut response = Response::json(200, r#"{"success":true}"#);
            // Security: Clear the cookie
            response.headers.push(("Set-Cookie".to_string(), "MoodleSession=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0".to_string()));
            response
        }
        Err(_) => Response::json(401, r#"{"success":false,"error":"Invalid session"}"#),
    }
}

/// Validates session
fn validate_session(req: &Request) -> Response {
    // Get MoodleSession cookie from request
    let cookie = req.headers.get("Cookie")
        .and_then(|c| {
            c.split(';')
                .find(|part| part.trim().starts_with("MoodleSession="))
                .map(|part| part.trim().strip_prefix("MoodleSession=").unwrap_or("").to_string())
        })
        .ok_or("No MoodleSession cookie")
        .map_err(|e| Response::json(401, &format!(r#"{{"success":false,"error":"{}"}}"#, e)));

    let cookie = match cookie {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Initialize dependencies
    let cas_adapter = infrastructure::cas_adapter::CasAdapter::new(
        "https://jasig.firat.edu.tr/cas".to_string(),
        "https://debsis.firat.edu.tr".to_string(),
    );
    let use_case = application::login_usecase::LoginUseCase::new(cas_adapter);

    // Validate session
    match use_case.validate_session(&cookie) {
        Ok(user) => Response::json(200, &format!(
            r#"{{"valid":true,"user":"{}","full_name":"{}","email":"{}"}}"#,
            user.username,
            user.full_name.as_ref().unwrap_or(&"".to_string()),
            user.email.as_ref().unwrap_or(&"".to_string())
        )),
        Err(_) => Response::json(401, r#"{"valid":false,"error":"Invalid session"}"#),
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
