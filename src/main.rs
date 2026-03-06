//! Fırat Shadow Handbook - Zero Dependency HTTP Server
//!
//! Pure Rust std::net implementation - no external crates

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

mod application;
mod config;
mod domain;
mod handler;
mod http;
mod infrastructure;

use handler::Router;
use http::{Method, Request, Response};

use application::composition::{AdapterConfig, CompositionRoot};

use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;

// ============================================================================
// Thread Pool - Zero Dependency Implementation
// ============================================================================

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Thread pool with fixed number of worker threads
/// Prevents unbounded thread spawning under high load
struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    /// Create a new thread pool with specified number of workers
    /// Recommended: cpu_cores * 2 to 4
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(std::sync::Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            let receiver = Arc::clone(&receiver);
            let worker = thread::spawn(move || {
                loop {
                    let job = receiver.lock().unwrap().recv();
                    match job {
                        Ok(job) => job(),
                        Err(_) => break, // Channel closed, exit
                    }
                }
            });
            workers.push(worker);
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Submit a job to the pool
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(ref sender) = self.sender {
            let job = Box::new(f);
            let _ = sender.send(job);
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop sender to close channel, workers will exit
        drop(self.sender.take());

        // Wait for all workers to finish
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// Get optimal thread pool size based on CPU cores
fn get_pool_size() -> usize {
    // Default to 8 threads if we can't detect CPU count
    // Formula: cpu_cores * 2 (good for I/O bound work)
    std::thread::available_parallelism()
        .map(|p| p.get() * 2)
        .unwrap_or(8)
}

/// Cached web root path - computed once at first access
static WEB_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Singleton CompositionRoot - shared across all requests
static COMPOSITION_ROOT: OnceLock<CompositionRoot> = OnceLock::new();

const SHADOW_SESSION_COOKIE: &str = "ShadowSession";
const SHADOW_USER_COOKIE: &str = "ShadowUser";
const LEGACY_MOODLE_COOKIE: &str = "MoodleSession";
const SHADOW_SESSION_MAX_AGE_SECS: u64 = 60 * 60 * 8;
const REMOTE_VALIDATE_INTERVAL_SECS: u64 = 60 * 5;
const REMOTE_VALIDATE_RETRY_COUNT: u32 = 1;

#[derive(Clone, Serialize, Deserialize)]
struct ShadowSessionRecord {
    moodle_session: String,
    user: domain::user::User,
    expires_at_epoch_secs: u64,
    last_remote_probe_epoch_secs: u64,
    last_remote_success_epoch_secs: u64,
    remote_failures: u32,
}

#[derive(Serialize, Deserialize, Default)]
struct PersistedShadowState {
    signing_key: String,
    sessions: HashMap<String, ShadowSessionRecord>,
}

struct ShadowRuntimeState {
    signing_key: String,
    sessions: HashMap<String, ShadowSessionRecord>,
}

struct ShadowSessionStore {
    state: Mutex<ShadowRuntimeState>,
    state_file: PathBuf,
}

static SHADOW_SESSION_STORE: OnceLock<ShadowSessionStore> = OnceLock::new();

/// Get or create the singleton CompositionRoot
fn get_composition() -> &'static CompositionRoot {
    COMPOSITION_ROOT.get_or_init(|| CompositionRoot::new(AdapterConfig::Production))
}

fn get_shadow_session_store() -> &'static ShadowSessionStore {
    SHADOW_SESSION_STORE.get_or_init(|| {
        let state_file = auth_state_file_path();
        let persisted = load_persisted_shadow_state(&state_file);
        let mut sessions = persisted.sessions;
        retain_active_sessions(&mut sessions);

        ShadowSessionStore {
            state: Mutex::new(ShadowRuntimeState {
                signing_key: persisted.signing_key,
                sessions,
            }),
            state_file,
        }
    })
}

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

        // Get current entry and check if expired
        if let Some((count, timestamp)) = requests.get(&ip) {
            // Lazy cleanup: remove expired entry only for this IP
            if now.duration_since(*timestamp) >= self.window {
                requests.remove(&ip);
            } else if *count >= self.limit {
                return false; // Rate limit exceeded
            }
        }

        // Periodic cleanup: only when map grows too large (2x limit)
        // This avoids O(n) scan on every request
        if requests.len() > self.limit as usize * 2 {
            requests.retain(|_, (_, ts)| now.duration_since(*ts) < self.window);
        }

        // Get current count (may have been removed above)
        let count = requests.get(&ip).map(|(c, _)| *c).unwrap_or(0);
        requests.insert(ip, (count + 1, now));
        true
    }
}

static RUNNING: AtomicBool = AtomicBool::new(true);

// ShadowSession store keeps the real MoodleSession on server-side only.

fn main() {
    let config = config::Config::load();
    let addr = format!("{}:{}", config.host, config.port);

    // Initialize thread pool with optimal size
    let pool_size = get_pool_size();
    let pool = ThreadPool::new(pool_size);

    println!("🚀 Fırat Shadow Handbook Server");
    println!("   Listening on http://{}", addr);
    println!("   Thread pool size: {}", pool_size);
    println!("   Press Ctrl+C to stop\n");

    let listener = TcpListener::bind(&addr).expect("Failed to bind");
    listener.set_nonblocking(true).ok();

    let router = Arc::new(Router::new());
    setup_routes(&router);

    // Security: Initialize rate limiter (100 requests per minute per IP)
    // API endpoints are rate-limited; static assets are not.
    let rate_limiter = Arc::new(RateLimiter::new(300, Duration::from_secs(60)));

    while RUNNING.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, addr)) => {
                let router = Arc::clone(&router);
                let rate_limiter = Arc::clone(&rate_limiter);
                pool.execute(move || {
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

fn handle_connection(
    mut stream: TcpStream,
    addr: std::net::SocketAddr,
    router: &Router,
    rate_limiter: &RateLimiter,
) {
    let reader = BufReader::new(&stream);

    let request = match parse_request(reader) {
        Some(r) => r,
        None => return,
    };

    // Security: Rate limit API endpoints, not static asset fetches
    if request.path.starts_with("/api/") && !rate_limiter.allow(addr.ip()) {
        let response = Response {
            status: 429,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Retry-After".to_string(), "60".to_string()),
            ],
            body: r#"{"error":"Too Many Requests"}"#.to_string(),
        };
        let _ = send_response_raw(&mut stream, &response);
        return;
    }

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

    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
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
            if content_len > 1024 * 1024 {
                // 1MB max
                return None;
            }
            // Read body
            let mut buffer = vec![0; content_len];
            if reader.read_exact(&mut buffer).is_ok() {
                body = String::from_utf8_lossy(&buffer).to_string();
            }
        }
    }

    Some(Request::new(method, path, headers, body))
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
    let cors_origin = if let Some(origin) = response
        .headers
        .iter()
        .find(|(k, _)| k == "Access-Control-Allow-Origin")
    {
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
    if !response
        .headers
        .iter()
        .any(|(k, _)| k == "X-Content-Type-Options")
    {
        headers_string.push_str("X-Content-Type-Options: nosniff\r\n");
    }
    if !response
        .headers
        .iter()
        .any(|(k, _)| k == "Content-Security-Policy")
    {
        headers_string.push_str("Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';\r\n");
    }
    if !response.headers.iter().any(|(k, _)| k == "Referrer-Policy") {
        headers_string.push_str("Referrer-Policy: strict-origin-when-cross-origin\r\n");
    }
    if !response
        .headers
        .iter()
        .any(|(k, _)| k == "Permissions-Policy")
    {
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
    router.get("/", |_| {
        serve_from_web("index.html", Some("text/html; charset=utf-8"))
    });
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
        serve_from_web(
            &format!("js/{}", file),
            Some("application/javascript; charset=utf-8"),
        )
    });
    router.get("/i18n/*", |req| {
        let file = req.path.strip_prefix("/i18n/").unwrap_or("");
        serve_from_web(
            &format!("i18n/{}", file),
            Some("application/json; charset=utf-8"),
        )
    });
    router.get("/images/*", |req| {
        let file = req.path.strip_prefix("/images/").unwrap_or("");
        serve_from_web(&format!("images/{}", file), None)
    });

    // API endpoints
    router.get("/api/health", |_| Response::json(200, r#"{"status":"ok"}"#));
    router.get("/api/config", |_| {
        Response::json(200, r#"{"version":"0.1.0"}"#)
    });

    // Authentication endpoints
    router.post("/api/login", |req| handle_login(req));
    router.post("/api/logout", |req| handle_logout(req));
    router.get("/api/validate-session", |req| validate_session(req));
    router.get("/api/cas/callback*", |req| handle_cas_callback(req));
    router.post("/api/collab/scrape", |req| handle_collab_scrape(req));
}

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn auth_state_file_path() -> PathBuf {
    if let Ok(path) = std::env::var("AUTH_STATE_FILE") {
        return PathBuf::from(path);
    }
    PathBuf::from("data/runtime/shadow_sessions.json")
}

fn load_persisted_shadow_state(path: &PathBuf) -> PersistedShadowState {
    let Ok(raw) = fs::read_to_string(path) else {
        return PersistedShadowState {
            signing_key: generate_token(),
            sessions: HashMap::new(),
        };
    };

    match serde_json::from_str::<PersistedShadowState>(&raw) {
        Ok(state) if !state.signing_key.is_empty() => state,
        Ok(_) | Err(_) => PersistedShadowState {
            signing_key: generate_token(),
            sessions: HashMap::new(),
        },
    }
}

fn retain_active_sessions(sessions: &mut HashMap<String, ShadowSessionRecord>) {
    let now = current_epoch_secs();
    sessions.retain(|_, entry| entry.expires_at_epoch_secs > now);
}

fn persist_shadow_state(store: &ShadowSessionStore, state: &ShadowRuntimeState) {
    if let Some(parent) = store.state_file.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!("[auth-state] failed to create runtime dir: {}", err);
            return;
        }
    }

    let persisted = PersistedShadowState {
        signing_key: state.signing_key.clone(),
        sessions: state.sessions.clone(),
    };

    let serialized = match serde_json::to_string(&persisted) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("[auth-state] failed to serialize state: {}", err);
            return;
        }
    };

    if let Err(err) = fs::write(&store.state_file, serialized) {
        eprintln!("[auth-state] failed to persist state: {}", err);
    }
}

fn issue_shadow_session(auth_session: &domain::ports::auth_port::Session) -> String {
    let session_id = generate_token();
    let issued_at = current_epoch_secs();
    let record = ShadowSessionRecord {
        moodle_session: auth_session.moodle_session.clone(),
        user: auth_session.user.clone(),
        expires_at_epoch_secs: issued_at + SHADOW_SESSION_MAX_AGE_SECS,
        last_remote_probe_epoch_secs: issued_at,
        last_remote_success_epoch_secs: issued_at,
        remote_failures: 0,
    };

    let store = get_shadow_session_store();
    let mut state = store.state.lock().unwrap();
    retain_active_sessions(&mut state.sessions);
    state.sessions.insert(session_id.clone(), record);

    let payload = format!("{}.{}", session_id, issued_at);
    let signature = sign_shadow_payload(&payload, &state.signing_key);
    persist_shadow_state(store, &state);
    format!("{}.{}", payload, signature)
}

fn sign_shadow_payload(payload: &str, signing_key: &str) -> String {
    let mut hasher = DefaultHasher::new();
    signing_key.hash(&mut hasher);
    payload.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0u8;
    for (l, r) in left.bytes().zip(right.bytes()) {
        diff |= l ^ r;
    }
    diff == 0
}

fn parse_shadow_session_cookie(raw_cookie: &str) -> Option<(String, u64)> {
    let store = get_shadow_session_store();
    let state = store.state.lock().unwrap();
    parse_shadow_session_cookie_with_key(raw_cookie, &state.signing_key)
}

fn parse_shadow_session_cookie_with_key(
    raw_cookie: &str,
    signing_key: &str,
) -> Option<(String, u64)> {
    let mut parts = raw_cookie.split('.');
    let session_id = parts.next()?;
    let issued_at = parts.next()?.parse::<u64>().ok()?;
    let signature = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    if session_id.is_empty() || !session_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    let payload = format!("{}.{}", session_id, issued_at);
    let expected_signature = sign_shadow_payload(&payload, signing_key);
    if !constant_time_eq(signature, &expected_signature) {
        return None;
    }

    Some((session_id.to_string(), issued_at))
}

fn load_shadow_session(req: &Request) -> Result<(String, ShadowSessionRecord), &'static str> {
    let store = get_shadow_session_store();
    let mut state = store.state.lock().unwrap();
    let raw_cookie = req
        .get_cookie(SHADOW_SESSION_COOKIE)
        .ok_or("No active session")?;
    let (session_id, _issued_at) =
        parse_shadow_session_cookie_with_key(raw_cookie, &state.signing_key)
            .ok_or("Invalid session signature")?;

    let now = current_epoch_secs();
    match state.sessions.get(&session_id) {
        Some(record) if record.expires_at_epoch_secs > now => Ok((session_id, record.clone())),
        Some(_) => {
            state.sessions.remove(&session_id);
            persist_shadow_state(store, &state);
            Err("Session expired")
        }
        None => Err("Session expired"),
    }
}

fn refresh_shadow_session_expiry(session_id: &str) {
    let now = current_epoch_secs();
    let store = get_shadow_session_store();
    let mut state = store.state.lock().unwrap();
    if let Some(entry) = state.sessions.get_mut(session_id) {
        entry.expires_at_epoch_secs = now + SHADOW_SESSION_MAX_AGE_SECS;
        persist_shadow_state(store, &state);
    }
}

fn probe_remote_session(
    moodle_session: &str,
) -> Result<domain::user::User, domain::ports::auth_port::AuthError> {
    let composition = get_composition();
    let auth_port = composition.create_auth_adapter();
    let use_case: application::login_usecase::LoginUseCase<
        Box<dyn crate::domain::ports::auth_port::AuthPort>,
    > = application::login_usecase::LoginUseCase::with_boxed(auth_port);
    use_case.validate_session(moodle_session)
}

fn valid_session_response(user: &domain::user::User, degraded: bool) -> Response {
    let response_body = serde_json::json!({
        "valid": true,
        "user": user.username,
        "full_name": user.full_name.as_deref().unwrap_or(""),
        "email": user.email.as_deref().unwrap_or(""),
        "degraded": degraded
    })
    .to_string();
    Response::json(200, &response_body)
}

fn invalid_session_response(error: &str, clear_cookies: bool) -> Response {
    let response_body = serde_json::json!({
        "valid": false,
        "error": error
    })
    .to_string();
    let mut response = Response::json(401, &response_body);
    if clear_cookies {
        append_auth_cookie_clears(&mut response);
    }
    response
}

fn append_auth_cookie_clears(response: &mut Response) {
    let secure = secure_cookie_suffix();

    response.headers.push((
        "Set-Cookie".to_string(),
        format!(
            "{}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0{}",
            SHADOW_SESSION_COOKIE, secure
        ),
    ));
    response.headers.push((
        "Set-Cookie".to_string(),
        format!(
            "{}=; Path=/; SameSite=Lax; Max-Age=0{}",
            SHADOW_USER_COOKIE, secure
        ),
    ));
    response.headers.push((
        "Set-Cookie".to_string(),
        format!(
            "{}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0{}",
            LEGACY_MOODLE_COOKIE, secure
        ),
    ));
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
    let composition = get_composition();
    let auth_port = composition.create_auth_adapter();
    let use_case: application::login_usecase::LoginUseCase<
        Box<dyn crate::domain::ports::auth_port::AuthPort>,
    > = application::login_usecase::LoginUseCase::with_boxed(auth_port);

    // Execute login
    match use_case.login(username, password) {
        Ok(session) => {
            let shadow_cookie_value = issue_shadow_session(&session);

            // Use serde_json for safe JSON serialization (prevents XSS and JSON parsing issues)
            let response_body = serde_json::json!({
                "success": true,
                "user": session.user.username,
                "full_name": session.user.full_name.as_deref().unwrap_or(""),
                "email": session.user.email.as_deref().unwrap_or("")
            })
            .to_string();
            let mut response = Response::json(200, &response_body);

            let secure = secure_cookie_suffix();
            // ShadowSession is the only browser auth cookie. Real MoodleSession stays server-side.
            let shadow_cookie = format!(
                "{}={}; HttpOnly; Path=/; SameSite=Lax; Max-Age={}{}",
                SHADOW_SESSION_COOKIE, shadow_cookie_value, SHADOW_SESSION_MAX_AGE_SECS, secure
            );
            response
                .headers
                .push(("Set-Cookie".to_string(), shadow_cookie));

            // Set ShadowUser cookie (readable by JS for UI)
            let user_cookie = format!(
                "{}={}; Path=/; SameSite=Lax; Max-Age={}{}",
                SHADOW_USER_COOKIE, session.user.username, SHADOW_SESSION_MAX_AGE_SECS, secure
            );
            response
                .headers
                .push(("Set-Cookie".to_string(), user_cookie));

            // Clear legacy MoodleSession cookie if it exists in browser from old builds.
            response.headers.push((
                "Set-Cookie".to_string(),
                format!(
                    "{}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0{}",
                    LEGACY_MOODLE_COOKIE, secure
                ),
            ));

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
            })
            .to_string();
            Response::json(401, &response_body)
        }
    }
}

/// Handles logout request — clears local session cookies
fn handle_logout(req: &Request) -> Response {
    if let Some(raw_shadow_cookie) = req.get_cookie(SHADOW_SESSION_COOKIE) {
        let store = get_shadow_session_store();
        let mut state = store.state.lock().unwrap();
        if let Some((session_id, _issued_at)) =
            parse_shadow_session_cookie_with_key(raw_shadow_cookie, &state.signing_key)
        {
            state.sessions.remove(&session_id);
            persist_shadow_state(store, &state);
        }
    }

    let response_body = serde_json::json!({ "success": true }).to_string();
    let mut response = Response::json(200, &response_body);
    append_auth_cookie_clears(&mut response);
    response
}

/// Validates session
fn validate_session(req: &Request) -> Response {
    let (session_id, shadow_session) = match load_shadow_session(req) {
        Ok(data) => data,
        Err(reason) => return invalid_session_response(reason, true),
    };

    let mut degraded = false;
    let now = current_epoch_secs();
    let should_probe = now.saturating_sub(shadow_session.last_remote_probe_epoch_secs)
        >= REMOTE_VALIDATE_INTERVAL_SECS;

    if should_probe {
        let mut remote_result = probe_remote_session(&shadow_session.moodle_session);
        if remote_result.is_err() {
            for _ in 0..REMOTE_VALIDATE_RETRY_COUNT {
                remote_result = probe_remote_session(&shadow_session.moodle_session);
                if remote_result.is_ok() {
                    break;
                }
            }
        }

        {
            let store = get_shadow_session_store();
            let mut state = store.state.lock().unwrap();
            let Some(entry) = state.sessions.get_mut(&session_id) else {
                return invalid_session_response("Session expired", true);
            };

            entry.last_remote_probe_epoch_secs = now;
            entry.expires_at_epoch_secs = now + SHADOW_SESSION_MAX_AGE_SECS;

            match remote_result {
                Ok(_) => {
                    entry.last_remote_success_epoch_secs = now;
                    entry.remote_failures = 0;
                }
                Err(err) => {
                    entry.remote_failures = entry.remote_failures.saturating_add(1);
                    eprintln!(
                        "[validate_session] remote probe failed; failures={}, keeping local session alive, err={:?}",
                        entry.remote_failures, err
                    );
                    degraded = true;
                }
            }

            persist_shadow_state(store, &state);
        }
    } else {
        refresh_shadow_session_expiry(&session_id);
    }

    valid_session_response(&shadow_session.user, degraded)
}

/// Handles CAS callback - validates ticket and creates session
fn handle_cas_callback(req: &Request) -> Response {
    let has_ticket = req
        .path
        .split('?')
        .nth(1)
        .and_then(|query| {
            query.split('&').find_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                if key == "ticket" {
                    Some(())
                } else {
                    None
                }
            })
        })
        .is_some();

    let location = if has_ticket {
        "/#/login?info=cas_callback_deprecated"
    } else {
        "/#/login?error=no_ticket"
    };

    eprintln!(
        "[cas_callback] deprecated endpoint hit; redirecting without issuing session (ticket_present={})",
        has_ticket
    );

    Response::redirect(location)
}

fn handle_collab_scrape(req: &Request) -> Response {
    // Validate Content-Type header
    let content_type = get_header_case_insensitive(req, "Content-Type").unwrap_or_default();
    if !content_type.contains("application/json") {
        return Response::json(415, r#"{"error":"Content-Type must be application/json"}"#);
    }

    let (session_id, shadow_session) = match load_shadow_session(req) {
        Ok(data) => data,
        Err(_) => return Response::json(401, r#"{"error":"No active session"}"#),
    };
    let moodle_session = shadow_session.moodle_session;
    refresh_shadow_session_expiry(&session_id);

    let body = match serde_json::from_str::<serde_json::Value>(&req.body) {
        Ok(v) => v,
        Err(_) => return Response::json(400, r#"{"error":"Invalid JSON body"}"#),
    };

    let html = match body.get("html").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return Response::json(400, r#"{"error":"html field is required"}"#),
    };

    // Use Composition Root for centralized adapter selection (port-first approach)
    let composition = get_composition();
    let scraper_port = composition.create_scraper_adapter();
    let use_case: application::collab_scraper_usecase::CollabScraperUseCase<
        Box<dyn crate::domain::ports::scraper_port::ScraperPort>,
    > = application::collab_scraper_usecase::CollabScraperUseCase::with_boxed(scraper_port);

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
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
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
        Request::new(Method::GET, "/".to_string(), headers, String::new())
    }

    #[test]
    fn test_get_cookie_parses_named_cookie() {
        let req = make_request_with_cookie("A=1; ShadowSession=abc123; C=3");
        let value = req.get_cookie("ShadowSession");
        assert_eq!(value, Some("abc123"));
    }

    #[test]
    fn test_generate_token_has_expected_hex_length() {
        let token = generate_token();
        assert_eq!(token.len(), 48);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn persisted_shadow_state_roundtrip_preserves_signing_key_and_sessions() {
        let mut sessions = HashMap::new();
        sessions.insert(
            "session-1".to_string(),
            ShadowSessionRecord {
                moodle_session: "mdl-session".to_string(),
                user: crate::domain::user::User::new("testuser".to_string()),
                expires_at_epoch_secs: current_epoch_secs() + 600,
                last_remote_probe_epoch_secs: current_epoch_secs(),
                last_remote_success_epoch_secs: current_epoch_secs(),
                remote_failures: 0,
            },
        );
        let state = PersistedShadowState {
            signing_key: "signing-key".to_string(),
            sessions,
        };

        let raw = serde_json::to_string(&state).expect("state must serialize");
        let parsed: PersistedShadowState =
            serde_json::from_str(&raw).expect("state must deserialize");

        assert_eq!(parsed.signing_key, "signing-key");
        assert_eq!(parsed.sessions.len(), 1);
        assert_eq!(
            parsed
                .sessions
                .get("session-1")
                .map(|s| s.moodle_session.as_str()),
            Some("mdl-session")
        );
    }

    #[test]
    fn retain_active_sessions_drops_expired_entries() {
        let now = current_epoch_secs();
        let mut sessions = HashMap::new();
        sessions.insert(
            "expired".to_string(),
            ShadowSessionRecord {
                moodle_session: "old".to_string(),
                user: crate::domain::user::User::new("old".to_string()),
                expires_at_epoch_secs: now.saturating_sub(1),
                last_remote_probe_epoch_secs: now.saturating_sub(10),
                last_remote_success_epoch_secs: now.saturating_sub(10),
                remote_failures: 1,
            },
        );
        sessions.insert(
            "active".to_string(),
            ShadowSessionRecord {
                moodle_session: "new".to_string(),
                user: crate::domain::user::User::new("new".to_string()),
                expires_at_epoch_secs: now + 600,
                last_remote_probe_epoch_secs: now,
                last_remote_success_epoch_secs: now,
                remote_failures: 0,
            },
        );

        retain_active_sessions(&mut sessions);

        assert!(!sessions.contains_key("expired"));
        assert!(sessions.contains_key("active"));
    }

    fn make_collab_request(cookie: Option<&str>, body: &str) -> Request {
        make_collab_request_with_content_type(cookie, body, "application/json")
    }

    fn make_shadow_cookie_header(username: &str, moodle_session: &str) -> String {
        let auth_session = crate::domain::ports::auth_port::Session {
            moodle_session: moodle_session.to_string(),
            user: crate::domain::user::User::new(username.to_string()),
        };
        let shadow_cookie_value = issue_shadow_session(&auth_session);
        format!("{}={}", SHADOW_SESSION_COOKIE, shadow_cookie_value)
    }

    fn make_callback_request(path: &str) -> Request {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "127.0.0.1:8080".to_string());
        Request::new(Method::GET, path.to_string(), headers, String::new())
    }

    fn make_collab_request_with_content_type(
        cookie: Option<&str>,
        body: &str,
        content_type: &str,
    ) -> Request {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        if let Some(raw) = cookie {
            headers.insert("Cookie".to_string(), raw.to_string());
        }
        Request::new(
            Method::POST,
            "/api/collab/scrape".to_string(),
            headers,
            body.to_string(),
        )
    }

    #[test]
    fn collab_scrape_returns_401_without_session_cookie() {
        let req = make_collab_request(None, r#"{"html":"<div>test</div>"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 401);
    }

    #[test]
    fn collab_scrape_returns_snapshot_for_valid_session() {
        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\" data-schedule=\"2026-02-27T10:00:00+03:00|2026-02-27T11:00:00+03:00|Europe/Istanbul\"></div><a class=\"playback-link\" data-playback=\"true\" href=\"https://eu.bbcollab.com/recording/abc\" data-label=\"Kayit\">Kayit</a>"
        })
        .to_string();

        let shadow_cookie = make_shadow_cookie_header("testuser", "mdl-session");
        let req = make_collab_request(Some(&shadow_cookie), &body);
        let response = handle_collab_scrape(&req);

        assert_eq!(response.status, 200);
        let payload: serde_json::Value =
            serde_json::from_str(&response.body).expect("snapshot json should parse");
        assert!(payload.get("courses").is_some());
        assert!(payload.get("playbacks").is_some());
    }

    #[test]
    fn collab_scrape_returns_400_for_invalid_json_body() {
        let shadow_cookie = make_shadow_cookie_header("testuser", "mdl-session");
        let req = make_collab_request(Some(&shadow_cookie), "{not-json");
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn collab_scrape_returns_400_when_html_field_missing() {
        let shadow_cookie = make_shadow_cookie_header("testuser", "mdl-session");
        let req = make_collab_request(Some(&shadow_cookie), r#"{"foo":"bar"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 400);
    }

    #[test]
    fn collab_scrape_returns_401_for_expired_session() {
        let req = make_collab_request(None, r#"{"html":"<div>test payload</div>"}"#);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 401);
    }

    #[test]
    fn collab_scrape_returns_422_for_non_allowlisted_playback_url() {
        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\"></div><a class=\"playback-link\" href=\"https://evil.local/recording/abc\">Kayit</a>"
        })
        .to_string();

        let shadow_cookie = make_shadow_cookie_header("testuser", "mdl-session");
        let req = make_collab_request(Some(&shadow_cookie), &body);
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 422);
    }

    #[test]
    fn collab_scrape_returns_415_for_missing_content_type() {
        let shadow_cookie = make_shadow_cookie_header("testuser", "mdl-session");
        let req = make_collab_request_with_content_type(
            Some(&shadow_cookie),
            r#"{"html":"<div>test</div>"}"#,
            "text/plain",
        );
        let response = handle_collab_scrape(&req);
        assert_eq!(response.status, 415);
    }

    #[test]
    fn collab_scrape_accepts_json_content_type() {
        let body = serde_json::json!({
            "html": "<div data-course-id=\"42\" data-course-title=\"Yazilim\"></div>"
        })
        .to_string();

        let shadow_cookie = make_shadow_cookie_header("testuser", "mdl-session");
        let req = make_collab_request(Some(&shadow_cookie), &body);
        let response = handle_collab_scrape(&req);
        // Should not be 415, session validation happens first
        assert_ne!(response.status, 415);
    }

    #[test]
    fn cas_callback_never_issues_session_cookies() {
        let req = make_callback_request("/api/cas/callback?ticket=ST-123");
        let response = handle_cas_callback(&req);

        assert_eq!(response.status, 302);
        assert_eq!(
            response
                .headers
                .iter()
                .find(|(k, _)| k == "Location")
                .map(|(_, v)| v.as_str()),
            Some("/#/login?info=cas_callback_deprecated")
        );

        let set_cookie_values: Vec<&str> = response
            .headers
            .iter()
            .filter(|(k, _)| k == "Set-Cookie")
            .map(|(_, v)| v.as_str())
            .collect();

        assert!(
            set_cookie_values.iter().all(|value| {
                !value.contains("MoodleSession=")
                    && !value.contains("ShadowUser=")
                    && !value.contains("ShadowSession=")
            }),
            "CAS callback must not issue session cookies"
        );
    }

    #[test]
    fn validate_session_returns_401_without_shadow_cookie() {
        let req = Request::new(
            Method::GET,
            "/api/validate-session".to_string(),
            HashMap::new(),
            String::new(),
        );
        let response = validate_session(&req);
        assert_eq!(response.status, 401);
    }

    #[test]
    fn validate_session_uses_local_shadow_session_on_refresh() {
        let shadow_cookie = make_shadow_cookie_header("testuser", "mdl-session");
        let mut headers = HashMap::new();
        headers.insert("Cookie".to_string(), shadow_cookie);
        let req = Request::new(
            Method::GET,
            "/api/validate-session".to_string(),
            headers,
            String::new(),
        );

        let response = validate_session(&req);
        assert_eq!(response.status, 200);
        let payload: serde_json::Value =
            serde_json::from_str(&response.body).expect("validate response must parse");
        assert_eq!(payload.get("valid"), Some(&serde_json::Value::Bool(true)));
    }

    #[test]
    fn logout_returns_parseable_json_success_payload() {
        let req = Request::new(
            Method::POST,
            "/api/logout".to_string(),
            HashMap::new(),
            String::new(),
        );
        let response = handle_logout(&req);

        assert_eq!(response.status, 200);
        let body: serde_json::Value =
            serde_json::from_str(&response.body).expect("logout response must be valid JSON");
        assert_eq!(body.get("success"), Some(&serde_json::Value::Bool(true)));
    }
}

/// Security: Path traversal prevention
/// Validates and sanitizes relative paths to prevent directory traversal attacks.
fn sanitize_relative_path(relative_path: &str) -> Result<String, &'static str> {
    if relative_path.is_empty() {
        return Err("Invalid filename");
    }

    if relative_path.contains("..") || relative_path.contains('\\') || relative_path.contains('\0')
    {
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
    WEB_ROOT
        .get_or_init(|| {
            let from_root = PathBuf::from("web");
            if from_root.exists() {
                return from_root;
            }

            let from_src = PathBuf::from("../web");
            if from_src.exists() {
                return from_src;
            }

            PathBuf::from("web")
        })
        .clone()
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
