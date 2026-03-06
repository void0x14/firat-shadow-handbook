//! CAS Authentication Adapter - Zero Dependency Implementation
//!
//! Implements CAS protocol for Fırat University's Jasig CAS server

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use crate::domain::ports::auth_port::{AuthError, AuthPort, Session};
use crate::domain::user::User;

const CONNECT_TIMEOUT_SECS: u64 = 10;
const MAX_REDIRECTS: usize = 5;

#[derive(Debug, Clone)]
struct HttpResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    set_cookies: Vec<String>,
    body: String,
}

trait CasTransport {
    fn send(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<&str>,
    ) -> Result<HttpResponse, AuthError>;
}

struct RustlsTransport {
    tls_config: Arc<ClientConfig>,
}

impl RustlsTransport {
    fn new() -> Self {
        let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        Self {
            tls_config: Arc::new(config),
        }
    }
}

impl CasTransport for RustlsTransport {
    fn send(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, String)],
        body: Option<&str>,
    ) -> Result<HttpResponse, AuthError> {
        let parsed = parse_https_url_parts(url)?;
        let socket = connect_with_timeout(&parsed.host, parsed.port)?;

        // Ensure the socket is explicitly set to blocking mode to avoid 'os error 11' (EAGAIN)
        // during rustls StreamOwned's TLS handshake which expects a blocking socket.
        socket
            .set_nonblocking(false)
            .map_err(|e| AuthError::NetworkError(format!("set_nonblocking failed: {}", e)))?;

        // Removed set_read_timeout and set_write_timeout from here.
        // Setting strict SO_RCVTIMEO or SO_SNDTIMEO on the underlying socket can cause
        // 'Resource temporarily unavailable' (EAGAIN) if the TLS handshake stalls or requires multiple round-trips.

        socket
            .set_nodelay(true)
            .map_err(|e| AuthError::NetworkError(format!("set_nodelay failed: {}", e)))?;

        let server_name = ServerName::try_from(parsed.host.clone())
            .map_err(|_| AuthError::NetworkError("Invalid TLS server name".to_string()))?;
        let conn = ClientConnection::new(Arc::clone(&self.tls_config), server_name)
            .map_err(|e| AuthError::NetworkError(format!("TLS connection init failed: {}", e)))?;

        let mut tls_stream = StreamOwned::new(conn, socket);
        let body_str = body.unwrap_or("");
        let mut request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: firat-shadow-handbook/0.1\r\nAccept: text/html,application/xhtml+xml,application/json\r\n",
            method, parsed.path, parsed.authority
        );

        for (k, v) in headers {
            request.push_str(&format!("{}: {}\r\n", k, v));
        }

        if !body_str.is_empty() {
            request.push_str(&format!("Content-Length: {}\r\n", body_str.len()));
        }

        request.push_str("\r\n");
        if !body_str.is_empty() {
            request.push_str(body_str);
        }

        tls_stream
            .write_all(request.as_bytes())
            .map_err(|e| AuthError::NetworkError(format!("TLS write failed: {}", e)))?;
        tls_stream
            .flush()
            .map_err(|e| AuthError::NetworkError(format!("TLS flush failed: {}", e)))?;

        let mut response_raw = String::new();
        let mut unexpected_eof = false;

        match tls_stream.read_to_string(&mut response_raw) {
            Ok(_) => {}
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("close_notify") || e.kind() == std::io::ErrorKind::UnexpectedEof
                {
                    // rustls UnexpectedEof behavior: connection dropped without proper TLS handshake closure.
                    // Instead of failing immediately, we mark that the connection was closed abruptly,
                    // and strictly rely on HTTP application-layer message framing (Zero Trust) to protect
                    // against truncation attacks as specified in the Rustls documentation.
                    unexpected_eof = true;
                } else {
                    return Err(AuthError::NetworkError(format!("TLS read failed: {}", e)));
                }
            }
        }

        parse_http_response(&response_raw, unexpected_eof)
    }
}

pub struct CasAdapter {
    cas_base_url: String,
    service_url: String,
}

impl CasAdapter {
    pub fn new(cas_base_url: String, service_url: String) -> Self {
        Self {
            cas_base_url: cas_base_url.trim_end_matches('/').to_string(),
            service_url,
        }
    }

    /// Public method to validate a CAS service ticket
    pub fn validate_ticket(&self, ticket: &str) -> Result<String, AuthError> {
        if ticket.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }

        let transport = RustlsTransport::new();
        self.validate_ticket_with_transport(ticket, &transport)
    }

    fn authenticate_with_transport<T: CasTransport>(
        &self,
        username: &str,
        password: &str,
        transport: &T,
    ) -> Result<Session, AuthError> {
        let login_url = format!(
            "{}/login?service={}",
            self.cas_base_url,
            url_encode(&self.service_url)
        );

        let mut cookie_jar: HashMap<String, String> = HashMap::new();

        eprintln!("[CAS] Step 1: GET {}", login_url);
        let login_page = transport.send("GET", &login_url, &[], None)?;
        if login_page.status_code != 200 {
            return Err(AuthError::CasServerError(format!(
                "CAS login page returned status {}",
                login_page.status_code
            )));
        }

        absorb_set_cookies(&mut cookie_jar, &login_page.set_cookies);
        let hidden_inputs = extract_hidden_inputs(&login_page.body);
        if hidden_inputs.is_empty() {
            return Err(AuthError::ParsingError(
                "CAS login form hidden fields not found".to_string(),
            ));
        }

        // POST to the same login URL (with service query) to preserve CAS flow.
        let post_url = login_url.clone();

        // Build dynamic form body: credentials + all hidden fields from CAS form.
        let mut form_pairs: Vec<(String, String)> = Vec::with_capacity(hidden_inputs.len() + 3);
        form_pairs.push(("username".to_string(), username.to_string()));
        form_pairs.push(("password".to_string(), password.to_string()));

        let mut has_event_id = false;
        for (name, value) in hidden_inputs {
            let lower = name.to_ascii_lowercase();
            if lower == "username" || lower == "password" {
                continue;
            }
            if name == "_eventId" {
                has_event_id = true;
            }
            form_pairs.push((name, value));
        }
        if !has_event_id {
            form_pairs.push(("_eventId".to_string(), "submit".to_string()));
        }

        // Deterministic ordering makes tests and debugging reproducible.
        form_pairs.sort_by(|a, b| a.0.cmp(&b.0));

        let mut form_body = String::with_capacity(512);
        for (idx, (key, value)) in form_pairs.iter().enumerate() {
            if idx > 0 {
                form_body.push('&');
            }
            write!(&mut form_body, "{}={}", url_encode(key), url_encode(value))
                .expect("form_body write should succeed");
        }

        let mut post_headers = vec![(
            "Content-Type",
            "application/x-www-form-urlencoded".to_string(),
        )];
        if let Some(cookie_header) = render_cookie_header(&cookie_jar) {
            post_headers.push(("Cookie", cookie_header));
        }

        eprintln!("[CAS] Step 2: POST {} (credentials)", post_url);
        let auth_response = transport.send("POST", &post_url, &post_headers, Some(&form_body))?;
        absorb_set_cookies(&mut cookie_jar, &auth_response.set_cookies);

        if auth_response.status_code == 200 {
            return Err(AuthError::InvalidCredentials);
        }

        if auth_response.status_code != 302 && auth_response.status_code != 303 {
            return Err(AuthError::CasServerError(format!(
                "CAS login submit returned status {}",
                auth_response.status_code
            )));
        }

        let mut next_url = auth_response
            .headers
            .get("location")
            .cloned()
            .ok_or_else(|| AuthError::CasServerError("Missing redirect location".to_string()))?;
        next_url = resolve_redirect_url(&post_url, &next_url)?;
        eprintln!("[CAS] Step 3: Auth success, redirect -> {}", next_url);

        for hop in 0..MAX_REDIRECTS {
            let mut headers = Vec::new();
            if let Some(cookie_header) = render_cookie_header(&cookie_jar) {
                headers.push(("Cookie", cookie_header));
            }

            eprintln!("[CAS] Step 4.{}: GET {}", hop, next_url);
            let response = transport.send("GET", &next_url, &headers, None)?;
            absorb_set_cookies(&mut cookie_jar, &response.set_cookies);
            eprintln!(
                "[CAS]   -> status={}, cookies={:?}",
                response.status_code,
                cookie_jar.keys().collect::<Vec<_>>()
            );

            if let Some(moodle) = cookie_jar.get("MoodleSession") {
                eprintln!("[CAS] SUCCESS: MoodleSession found");
                return Ok(Session {
                    moodle_session: moodle.clone(),
                    user: User::new(username.to_string()),
                });
            }

            if response.status_code == 302 || response.status_code == 303 {
                let location = response.headers.get("location").cloned().ok_or_else(|| {
                    AuthError::CasServerError("Redirect response missing location".to_string())
                })?;
                next_url = resolve_redirect_url(&next_url, &location)?;
                continue;
            }

            if response.status_code != 200 {
                return Err(AuthError::CasServerError(format!(
                    "Service callback returned status {}",
                    response.status_code
                )));
            }

            break;
        }

        Err(AuthError::ParsingError(
            "MoodleSession cookie not found after CAS flow".to_string(),
        ))
    }

    fn validate_session_with_transport<T: CasTransport>(
        &self,
        cookie: &str,
        transport: &T,
    ) -> Result<User, AuthError> {
        if cookie.is_empty() {
            return Err(AuthError::InvalidSession);
        }

        // Debsis base URL - service_url contains query params for CAS login
        // but for session validation we need the base URL
        let base_url = if self.service_url.contains("/login/index.php") {
            "https://debsis.firat.edu.tr".to_string()
        } else {
            self.service_url.trim_end_matches('/').to_string()
        };
        let expected_host = parse_https_url_parts(&base_url)?.host.to_ascii_lowercase();
        let probe_url = format!("{}/my/", base_url);
        let headers = [("Cookie", format!("MoodleSession={}", cookie))];
        let response = transport.send("GET", &probe_url, &headers, None)?;
        let location = response
            .headers
            .get("location")
            .map(String::as_str)
            .unwrap_or("-");
        let body_snippet = response
            .body
            .chars()
            .take(140)
            .collect::<String>()
            .replace('\n', " ")
            .replace('\r', " ");
        eprintln!(
            "[CAS][validate] probe={} status={} location={} body_snippet=\"{}\"",
            probe_url, response.status_code, location, body_snippet
        );

        match response.status_code {
            200 => {
                let body = response.body.to_ascii_lowercase();
                if body.contains("cas/login") && body.contains("username") {
                    return Err(AuthError::InvalidSession);
                }
                Ok(User::new("authenticated-user".to_string()))
            }
            302 | 303 => {
                let location = response.headers.get("location").cloned().ok_or_else(|| {
                    AuthError::CasServerError("Missing redirect location".to_string())
                })?;

                let resolved_location = match resolve_redirect_url(&probe_url, &location) {
                    Ok(url) => url,
                    Err(_) => return Err(AuthError::InvalidSession),
                };

                if !is_allowlisted_validation_redirect(&resolved_location, &expected_host) {
                    return Err(AuthError::InvalidSession);
                }

                Ok(User::new("authenticated-user".to_string()))
            }
            401 | 403 => Err(AuthError::InvalidSession),
            other => Err(AuthError::CasServerError(format!(
                "Session validation failed with status {}",
                other
            ))),
        }
    }

    fn logout_with_transport<T: CasTransport>(
        &self,
        _cookie: &str,
        _transport: &T,
    ) -> Result<(), AuthError> {
        // Local logout only - CAS'a logout atılmıyor
        // Cookie temizliği main.rs handle_logout tarafından yapılıyor
        Ok(())
    }

    /// Validate a CAS service ticket (ST) and return the authenticated user
    fn validate_ticket_with_transport<T: CasTransport>(
        &self,
        ticket: &str,
        transport: &T,
    ) -> Result<String, AuthError> {
        if ticket.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }

        // CAS serviceValidate endpoint
        let validate_url = format!(
            "{}/serviceValidate?service={}&ticket={}",
            self.cas_base_url,
            url_encode(&self.service_url),
            url_encode(ticket)
        );

        let response = transport.send("GET", &validate_url, &[], None)?;

        if response.status_code != 200 {
            return Err(AuthError::CasServerError(format!(
                "CAS validate returned status {}",
                response.status_code
            )));
        }

        // Parse CAS XML response
        // Success: <cas:serviceResponse><cas:authenticationSuccess><cas:user>username</cas:user>...</cas:authenticationSuccess></cas:serviceResponse>
        // Failure: <cas:serviceResponse><cas:authenticationFailure code="...">...</cas:authenticationFailure></cas:serviceResponse>
        let body = &response.body;

        if body.contains("<cas:authenticationSuccess>") || body.contains("<authenticationSuccess>")
        {
            // Extract username from response
            if let Some(user) = extract_cas_username(body) {
                return Ok(user);
            }
        }

        if body.contains("<cas:authenticationFailure>") || body.contains("<authenticationFailure>")
        {
            return Err(AuthError::InvalidCredentials);
        }

        Err(AuthError::ParsingError(
            "Could not parse CAS validation response".to_string(),
        ))
    }
}

impl AuthPort for CasAdapter {
    fn authenticate(&self, username: &str, password: &str) -> Result<Session, AuthError> {
        if username.is_empty() || password.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }

        let transport = RustlsTransport::new();
        self.authenticate_with_transport(username, password, &transport)
    }

    fn validate_session(&self, cookie: &str) -> Result<User, AuthError> {
        let transport = RustlsTransport::new();
        self.validate_session_with_transport(cookie, &transport)
    }

    fn logout(&self, cookie: &str) -> Result<(), AuthError> {
        let transport = RustlsTransport::new();
        self.logout_with_transport(cookie, &transport)
    }
}

struct ParsedHttpsUrl {
    host: String,
    authority: String,
    port: u16,
    path: String,
}

fn parse_https_url_parts(url: &str) -> Result<ParsedHttpsUrl, AuthError> {
    if !url.starts_with("https://") {
        return Err(AuthError::NetworkError(format!(
            "Only https URLs are supported: {}",
            url
        )));
    }

    let without_scheme = &url["https://".len()..];

    // Split authority from path+query. The authority ends at first '/' or '?'
    let (authority, path) = if let Some(slash_pos) = without_scheme.find('/') {
        (
            &without_scheme[..slash_pos],
            format!("/{}", &without_scheme[slash_pos + 1..]),
        )
    } else if let Some(q_pos) = without_scheme.find('?') {
        (
            &without_scheme[..q_pos],
            format!("/?{}", &without_scheme[q_pos + 1..]),
        )
    } else {
        (without_scheme, "/".to_string())
    };

    let (host, port) = if let Some((h, p)) = authority.rsplit_once(':') {
        match p.parse::<u16>() {
            Ok(parsed) => (h.to_string(), parsed),
            Err(_) => (authority.to_string(), 443),
        }
    } else {
        (authority.to_string(), 443)
    };

    Ok(ParsedHttpsUrl {
        host,
        authority: authority.to_string(),
        port,
        path,
    })
}

fn connect_with_timeout(host: &str, port: u16) -> Result<TcpStream, AuthError> {
    let mut last_error = None;
    for addr in (host, port)
        .to_socket_addrs()
        .map_err(|e| AuthError::NetworkError(format!("DNS resolve failed: {}", e)))?
    {
        match TcpStream::connect_timeout(&addr, Duration::from_secs(CONNECT_TIMEOUT_SECS)) {
            Ok(stream) => return Ok(stream),
            Err(err) => last_error = Some(err),
        }
    }
    Err(AuthError::NetworkError(format!(
        "TCP connect timeout/failure: {}",
        last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "no address resolved".to_string())
    )))
}

fn resolve_redirect_url(current_url: &str, location: &str) -> Result<String, AuthError> {
    if location.starts_with("https://") {
        return Ok(location.to_string());
    }

    // Accept http:// redirects but upgrade to https:// for security
    if location.starts_with("http://") {
        let upgraded = format!("https://{}", &location["http://".len()..]);
        eprintln!("[CAS] Upgrading http -> https: {}", upgraded);
        return Ok(upgraded);
    }

    if !location.starts_with('/') {
        return Err(AuthError::CasServerError(format!(
            "Unsupported redirect location: {}",
            location
        )));
    }

    let parsed = parse_https_url_parts(current_url)?;
    Ok(format!("https://{}{}", parsed.authority, location))
}

fn is_allowlisted_validation_redirect(url: &str, expected_host: &str) -> bool {
    let parsed = match parse_https_url_parts(url) {
        Ok(parts) => parts,
        Err(_) => return false,
    };

    let host = parsed.host.to_ascii_lowercase();
    if host != expected_host {
        return false;
    }

    let path = parsed.path.to_ascii_lowercase();
    let moodle_login_transition = path.starts_with("/login/index.php?testsession=")
        || path.starts_with("/login/index.php?authcas=cas")
        || path.starts_with("/login/index.php?redirect=0");

    if path.starts_with("/cas/login") {
        return false;
    }

    if path.starts_with("/login/index.php") && !moodle_login_transition {
        return false;
    }

    path == "/"
        || path.starts_with("/?")
        || moodle_login_transition
        || path.starts_with("/my")
        || path.starts_with("/course")
        || path.starts_with("/calendar")
        || path.starts_with("/user")
        || path.starts_with("/mod")
        || path.starts_with("/message")
        || path.starts_with("/grade")
        || path.starts_with("/auth/cas/")
}

fn parse_http_response(raw: &str, unexpected_eof: bool) -> Result<HttpResponse, AuthError> {
    let (head, body) = raw.split_once("\r\n\r\n").ok_or_else(|| {
        AuthError::ParsingError("Invalid HTTP response: missing header/body separator".to_string())
    })?;

    let mut lines = head.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| AuthError::ParsingError("Missing HTTP status line".to_string()))?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| AuthError::ParsingError("Missing HTTP status code".to_string()))?
        .parse::<u16>()
        .map_err(|_| AuthError::ParsingError("Invalid HTTP status code".to_string()))?;

    let mut headers = HashMap::new();
    let mut set_cookies = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_lowercase();
            let value = v.trim().to_string();
            if key == "set-cookie" {
                set_cookies.push(value.clone());
            }
            headers.insert(key, value);
        }
    }

    // Zero-Trust Validation: Mitigate TLS Truncation Attacks
    // If we received an UnexpectedEof, we MUST verify HTTP framing to ensure the body wasn't truncated.
    // See: https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof
    if unexpected_eof {
        let mut is_completed = false;

        if let Some(content_length) = headers.get("content-length") {
            if let Ok(len) = content_length.parse::<usize>() {
                if body.len() >= len {
                    is_completed = true;
                }
            }
        } else if let Some(transfer_encoding) = headers.get("transfer-encoding") {
            if transfer_encoding.to_lowercase().contains("chunked") {
                if body.ends_with("\r\n0\r\n\r\n") || body.ends_with("\n0\n\n") {
                    is_completed = true;
                }
            }
        } else {
            // Some redirect responses (302) legitly have zero body and no Content-Length
            if body.is_empty() {
                is_completed = true;
            }
        }

        if !is_completed {
            return Err(AuthError::NetworkError(
                "Truncated HTTP response detected (UnexpectedEof without valid message framing)"
                    .to_string(),
            ));
        }
    }

    Ok(HttpResponse {
        status_code,
        headers,
        set_cookies,
        body: body.to_string(),
    })
}

#[cfg(test)]
fn extract_hidden_fields(html: &str) -> Result<(String, String), AuthError> {
    let lt = extract_hidden_input_value(html, "lt")
        .ok_or_else(|| AuthError::ParsingError("LT hidden field missing".to_string()))?;
    let execution = extract_hidden_input_value(html, "execution")
        .ok_or_else(|| AuthError::ParsingError("execution hidden field missing".to_string()))?;
    Ok((lt, execution))
}

#[cfg(test)]
fn extract_hidden_input_value(html: &str, input_name: &str) -> Option<String> {
    let mut cursor = 0usize;
    while let Some(input_start_rel) = html[cursor..].find("<input") {
        let input_start = cursor + input_start_rel;
        let after_input = &html[input_start..];
        let end_rel = after_input.find('>')?;
        let tag = &after_input[..end_rel];
        let name = parse_attribute(tag, "name");
        if name.as_deref() == Some(input_name) {
            return parse_attribute(tag, "value");
        }
        cursor = input_start + end_rel + 1;
    }
    None
}

fn extract_hidden_inputs(html: &str) -> Vec<(String, String)> {
    let mut cursor = 0usize;
    let mut inputs = Vec::new();

    while let Some(input_start_rel) = html[cursor..].find("<input") {
        let input_start = cursor + input_start_rel;
        let after_input = &html[input_start..];
        let Some(end_rel) = after_input.find('>') else {
            break;
        };
        let tag = &after_input[..end_rel];

        let input_type = parse_attribute(tag, "type")
            .map(|v| v.to_ascii_lowercase())
            .unwrap_or_default();
        if input_type == "hidden" {
            if let (Some(name), Some(value)) =
                (parse_attribute(tag, "name"), parse_attribute(tag, "value"))
            {
                inputs.push((name, value));
            }
        }

        cursor = input_start + end_rel + 1;
    }

    inputs
}

fn parse_attribute(tag: &str, attribute: &str) -> Option<String> {
    let mut cursor = 0usize;
    while let Some(rel) = tag[cursor..].find(attribute) {
        let start = cursor + rel;
        let before = if start == 0 {
            ' '
        } else {
            tag.as_bytes()[start - 1] as char
        };
        if before.is_ascii_alphanumeric() || before == '-' || before == '_' {
            cursor = start + attribute.len();
            continue;
        }

        let mut idx = start + attribute.len();
        while idx < tag.len() && tag.as_bytes()[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= tag.len() || tag.as_bytes()[idx] != b'=' {
            cursor = start + attribute.len();
            continue;
        }
        idx += 1;
        while idx < tag.len() && tag.as_bytes()[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= tag.len() {
            return None;
        }

        let quote = tag.as_bytes()[idx] as char;
        if quote == '"' || quote == '\'' {
            idx += 1;
            let rest = tag.get(idx..)?;
            let end = rest.find(quote)?;
            return rest.get(..end).map(|s| s.to_string());
        }

        let rest = tag.get(idx..)?;
        let end = rest
            .find(|c: char| c.is_ascii_whitespace() || c == '>')
            .unwrap_or(rest.len());
        return rest.get(..end).map(|s| s.to_string());
    }
    None
}

fn absorb_set_cookies(cookie_jar: &mut HashMap<String, String>, set_cookie_headers: &[String]) {
    for header in set_cookie_headers {
        if let Some((name, value)) = parse_cookie(header) {
            cookie_jar.insert(name, value);
        }
    }
}

fn parse_cookie(set_cookie_header: &str) -> Option<(String, String)> {
    let (name, value) = set_cookie_header.split_once('=')?;
    let value = value.split(';').next()?.trim().to_string();
    Some((name.trim().to_string(), value))
}

fn render_cookie_header(cookie_jar: &HashMap<String, String>) -> Option<String> {
    if cookie_jar.is_empty() {
        return None;
    }
    let mut entries: Vec<String> = cookie_jar
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    entries.sort();
    Some(entries.join("; "))
}

/// Extract username from CAS serviceValidate XML response
fn extract_cas_username(xml: &str) -> Option<String> {
    // Look for <cas:user>username</cas:user> or <user>username</user>
    let patterns = ["<cas:user>", "<user>"];

    for pattern in &patterns {
        if let Some(start) = xml.find(pattern) {
            let after_start = &xml[start + pattern.len()..];
            let end_pattern = if pattern == &"<cas:user>" {
                "</cas:user>"
            } else {
                "</user>"
            };
            if let Some(end) = after_start.find(end_pattern) {
                return Some(after_start[..end].to_string());
            }
        }
    }

    None
}

fn url_encode(input: &str) -> String {
    let mut output = String::new();
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            output.push(byte as char);
        } else {
            output.push_str(&format!("%{:02X}", byte));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct MockTransport {
        responses: RefCell<VecDeque<Result<HttpResponse, AuthError>>>,
    }

    impl MockTransport {
        fn new(responses: Vec<Result<HttpResponse, AuthError>>) -> Self {
            Self {
                responses: RefCell::new(VecDeque::from(responses)),
            }
        }
    }

    impl CasTransport for MockTransport {
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: &[(&str, String)],
            _body: Option<&str>,
        ) -> Result<HttpResponse, AuthError> {
            self.responses.borrow_mut().pop_front().unwrap_or_else(|| {
                Err(AuthError::CasServerError(
                    "missing mock response".to_string(),
                ))
            })
        }
    }

    fn response(
        status: u16,
        headers: &[(&str, &str)],
        set_cookies: &[&str],
        body: &str,
    ) -> HttpResponse {
        HttpResponse {
            status_code: status,
            headers: headers
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            set_cookies: set_cookies.iter().map(|c| c.to_string()).collect(),
            body: body.to_string(),
        }
    }

    #[test]
    fn parse_hidden_fields_success() {
        let html = r#"<input type="hidden" name="lt" value="lt-123"><input name="execution" value="e1s1">"#;
        let (lt, execution) = extract_hidden_fields(html).expect("hidden fields should parse");
        assert_eq!(lt, "lt-123");
        assert_eq!(execution, "e1s1");
    }

    #[test]
    fn parse_hidden_fields_missing_returns_error() {
        let html = "<html><body>no hidden input</body></html>";
        let err = extract_hidden_fields(html).expect_err("missing fields should fail");
        assert!(matches!(err, AuthError::ParsingError(_)));
    }

    #[test]
    fn extract_hidden_inputs_supports_spaces_around_equal_sign() {
        let html = r#"<input type = "hidden" name = "execution" value = "e1s1"><input type='hidden' name='lt' value='lt-1'>"#;
        let inputs = extract_hidden_inputs(html);
        assert!(inputs.iter().any(|(k, v)| k == "execution" && v == "e1s1"));
        assert!(inputs.iter().any(|(k, v)| k == "lt" && v == "lt-1"));
    }

    #[test]
    fn parse_cookie_extracts_value() {
        let cookie = parse_cookie("MoodleSession=abc123; Path=/; HttpOnly")
            .expect("cookie parsing should work");
        assert_eq!(cookie.0, "MoodleSession");
        assert_eq!(cookie.1, "abc123");
    }

    #[test]
    fn authenticate_flow_success_with_mock_transport() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![
            Ok(response(
                200,
                &[],
                &["JSESSIONID=jsess-1; Path=/; HttpOnly"],
                r#"<input type="hidden" name="lt" value="lt-1"><input type="hidden" name="execution" value="e1s1">"#,
            )),
            Ok(response(
                302,
                &[(
                    "location",
                    "https://debsis.firat.edu.tr/login/index.php?ticket=ST-1",
                )],
                &[],
                "",
            )),
            Ok(response(
                200,
                &[],
                &["MoodleSession=mdl-xyz; Path=/; HttpOnly"],
                "",
            )),
        ]);

        let session = adapter
            .authenticate_with_transport("testuser", "testpass", &transport)
            .expect("auth flow should succeed");

        assert_eq!(session.moodle_session, "mdl-xyz");
        assert_eq!(session.user.username, "testuser");
    }

    #[test]
    fn authenticate_flow_success_without_lt_hidden_field() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr/login/index.php?authCAS=CAS".to_string(),
        );
        let transport = MockTransport::new(vec![
            Ok(response(
                200,
                &[],
                &["JSESSIONID=jsess-1; Path=/; HttpOnly"],
                r#"<form><input type="hidden" name="execution" value="e1s1"><input type="hidden" name="geolocation" value=""></form>"#,
            )),
            Ok(response(
                302,
                &[(
                    "location",
                    "https://debsis.firat.edu.tr/login/index.php?authCAS=CAS&ticket=ST-1",
                )],
                &[],
                "",
            )),
            Ok(response(
                200,
                &[],
                &["MoodleSession=mdl-xyz; Path=/; HttpOnly"],
                "",
            )),
        ]);

        let session = adapter
            .authenticate_with_transport("testuser", "testpass", &transport)
            .expect("auth flow should succeed even if lt is absent");

        assert_eq!(session.moodle_session, "mdl-xyz");
        assert_eq!(session.user.username, "testuser");
    }

    #[test]
    fn authenticate_fails_on_invalid_credentials_status_200_after_post() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![
            Ok(response(
                200,
                &[],
                &["JSESSIONID=jsess-1; Path=/"],
                r#"<input type="hidden" name="lt" value="lt-1"><input type="hidden" name="execution" value="e1s1">"#,
            )),
            Ok(response(200, &[], &[], "<html>login failed</html>")),
        ]);

        let err = adapter
            .authenticate_with_transport("bad", "bad", &transport)
            .expect_err("auth should fail");
        assert!(matches!(err, AuthError::InvalidCredentials));
    }

    #[test]
    fn parse_http_response_extracts_status_headers_and_body() {
        let raw = "HTTP/1.1 302 Found\r\nLocation: https://example.com\r\nSet-Cookie: A=1; Path=/\r\n\r\n<body>ok</body>";
        let parsed = parse_http_response(raw, false).expect("response should parse");
        assert_eq!(parsed.status_code, 302);
        assert_eq!(
            parsed.headers.get("location").map(String::as_str),
            Some("https://example.com")
        );
        assert_eq!(parsed.set_cookies.len(), 1);
        assert_eq!(parsed.body, "<body>ok</body>");
    }

    #[test]
    fn resolve_redirect_url_supports_relative_locations() {
        let resolved = resolve_redirect_url(
            "https://jasig.firat.edu.tr/cas/login",
            "/cas/login?service=https%3A%2F%2Fdebsis.firat.edu.tr",
        )
        .expect("relative redirect should resolve");

        assert_eq!(
            resolved,
            "https://jasig.firat.edu.tr/cas/login?service=https%3A%2F%2Fdebsis.firat.edu.tr"
        );
    }

    #[test]
    fn resolve_redirect_url_rejects_non_https_non_relative_locations() {
        // http:// is now accepted (upgraded to https://)
        let upgraded = resolve_redirect_url(
            "https://jasig.firat.edu.tr/cas/login",
            "http://debsis.firat.edu.tr/my/",
        )
        .expect("http redirect should be upgraded");
        assert_eq!(upgraded, "https://debsis.firat.edu.tr/my/");

        // ftp:// and other schemes are rejected
        let err = resolve_redirect_url("https://jasig.firat.edu.tr/cas/login", "ftp://evil.local")
            .expect_err("unsupported scheme must be rejected");
        assert!(matches!(err, AuthError::CasServerError(_)));
    }

    #[test]
    fn extract_hidden_input_value_supports_single_quote_and_reordered_attributes() {
        let html =
            r#"<input value='e1s1' type="hidden" name='execution'><input value="lt-9" name="lt">"#;
        let lt = extract_hidden_input_value(html, "lt").expect("lt should parse");
        let execution =
            extract_hidden_input_value(html, "execution").expect("execution should parse");
        assert_eq!(lt, "lt-9");
        assert_eq!(execution, "e1s1");
    }

    #[test]
    fn validate_session_with_transport_returns_invalid_for_cas_redirect() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![Ok(response(
            302,
            &[(
                "location",
                "https://jasig.firat.edu.tr/cas/login?service=abc",
            )],
            &[],
            "",
        ))]);

        let err = adapter
            .validate_session_with_transport("cookie", &transport)
            .expect_err("redirect to cas login must invalidate session");
        assert!(matches!(err, AuthError::InvalidSession));
    }

    #[test]
    fn validate_session_with_transport_accepts_authenticated_page() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![Ok(response(
            200,
            &[],
            &[],
            "<html><body>Dashboard</body></html>",
        ))]);

        let user = adapter
            .validate_session_with_transport("cookie", &transport)
            .expect("authenticated content should pass");
        assert_eq!(user.username, "authenticated-user");
    }

    #[test]
    fn validate_session_with_transport_accepts_allowlisted_redirect() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![Ok(response(
            302,
            &[("location", "https://debsis.firat.edu.tr/my/?lang=tr")],
            &[],
            "",
        ))]);

        let user = adapter
            .validate_session_with_transport("cookie", &transport)
            .expect("allowlisted redirect should keep session valid");
        assert_eq!(user.username, "authenticated-user");
    }

    #[test]
    fn validate_session_with_transport_accepts_moodle_login_transition_redirect() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![Ok(response(
            302,
            &[(
                "location",
                "https://debsis.firat.edu.tr/login/index.php?testsession=1",
            )],
            &[],
            "",
        ))]);

        let user = adapter
            .validate_session_with_transport("cookie", &transport)
            .expect("known login transition redirect should keep session valid");
        assert_eq!(user.username, "authenticated-user");
    }

    #[test]
    fn validate_session_with_transport_rejects_unknown_debsis_redirect_path() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![Ok(response(
            302,
            &[(
                "location",
                "https://debsis.firat.edu.tr/unknown-redirect-target",
            )],
            &[],
            "",
        ))]);

        let err = adapter
            .validate_session_with_transport("cookie", &transport)
            .expect_err("unknown debsis redirect must invalidate session");
        assert!(matches!(err, AuthError::InvalidSession));
    }

    #[test]
    fn validate_session_with_transport_rejects_unknown_relative_redirect_path() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![Ok(response(
            303,
            &[("location", "/unknown-relative")],
            &[],
            "",
        ))]);

        let err = adapter
            .validate_session_with_transport("cookie", &transport)
            .expect_err("unknown relative redirect must invalidate session");
        assert!(matches!(err, AuthError::InvalidSession));
    }

    #[test]
    fn logout_with_transport_always_succeeds_local_only() {
        // Local logout her zaman başarılı - CAS'a istek atılmıyor
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr/login/index.php?authCAS=CAS".to_string(),
        );
        let transport = MockTransport::new(vec![]); // Hiç istek yapılmayacak

        let result = adapter.logout_with_transport("cookie", &transport);
        assert!(result.is_ok(), "Local logout should always succeed");
    }
}
