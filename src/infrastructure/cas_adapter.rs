// Infrastructure: CAS Adapter (real HTTPS CAS flow with rustls)

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;


use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use crate::domain::ports::auth_port::{AuthError, AuthPort, Session};
use crate::domain::user::User;

const CONNECT_TIMEOUT_SECS: u64 = 10;
const READ_TIMEOUT_SECS: u64 = 15;
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
        socket
            .set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECS)))
            .map_err(|e| AuthError::NetworkError(format!("set_read_timeout failed: {}", e)))?;
        socket
            .set_write_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECS)))
            .map_err(|e| AuthError::NetworkError(format!("set_write_timeout failed: {}", e)))?;
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
        tls_stream
            .read_to_string(&mut response_raw)
            .map_err(|e| AuthError::NetworkError(format!("TLS read failed: {}", e)))?;

        parse_http_response(&response_raw)
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

        let login_page = transport.send("GET", &login_url, &[], None)?;
        if login_page.status_code != 200 {
            return Err(AuthError::CasServerError(format!(
                "CAS login page returned status {}",
                login_page.status_code
            )));
        }

        absorb_set_cookies(&mut cookie_jar, &login_page.set_cookies);
        let (lt, execution) = extract_hidden_fields(&login_page.body)?;

        let post_url = format!("{}/login", self.cas_base_url);
        let form_body = format!(
            "username={}&password={}&lt={}&execution={}&_eventId=submit",
            url_encode(username),
            url_encode(password),
            url_encode(&lt),
            url_encode(&execution)
        );

        let mut post_headers = vec![(
            "Content-Type",
            "application/x-www-form-urlencoded".to_string(),
        )];
        if let Some(cookie_header) = render_cookie_header(&cookie_jar) {
            post_headers.push(("Cookie", cookie_header));
        }

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

        for _ in 0..MAX_REDIRECTS {
            let mut headers = Vec::new();
            if let Some(cookie_header) = render_cookie_header(&cookie_jar) {
                headers.push(("Cookie", cookie_header));
            }

            let response = transport.send("GET", &next_url, &headers, None)?;
            absorb_set_cookies(&mut cookie_jar, &response.set_cookies);

            if let Some(moodle) = cookie_jar.get("MoodleSession") {
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

        let probe_url = format!("{}/my/", self.service_url.trim_end_matches('/'));
        let headers = [("Cookie", format!("MoodleSession={}", cookie))];
        let response = transport.send("GET", &probe_url, &headers, None)?;

        match response.status_code {
            200 => {
                let body = response.body.to_ascii_lowercase();
                if body.contains("cas/login") && body.contains("username") {
                    return Err(AuthError::InvalidSession);
                }
                Ok(User::new("authenticated-user".to_string()))
            }
            302 | 303 => {
                let location = response
                    .headers
                    .get("location")
                    .cloned()
                    .ok_or_else(|| AuthError::CasServerError("Missing redirect location".to_string()))?;
                let lower = location.to_ascii_lowercase();
                if lower.contains("/cas/login") || lower.contains("service=") {
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
        cookie: &str,
        transport: &T,
    ) -> Result<(), AuthError> {
        if cookie.is_empty() {
            return Err(AuthError::InvalidSession);
        }

        let logout_url = format!(
            "{}/logout?service={}",
            self.cas_base_url,
            url_encode(&self.service_url)
        );
        let headers = [("Cookie", format!("MoodleSession={}", cookie))];
        let response = transport.send("GET", &logout_url, &headers, None)?;
        match response.status_code {
            200 | 302 | 303 => Ok(()),
            other => Err(AuthError::CasServerError(format!(
                "CAS logout failed with status {}",
                other
            ))),
        }
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
    let mut parts = without_scheme.splitn(2, '/');
    let authority = parts
        .next()
        .ok_or_else(|| AuthError::NetworkError("Invalid URL host".to_string()))?;
    let path = format!("/{}", parts.next().unwrap_or(""));

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

    if !location.starts_with('/') {
        return Err(AuthError::CasServerError(format!(
            "Unsupported redirect location: {}",
            location
        )));
    }

    let parsed = parse_https_url_parts(current_url)?;
    Ok(format!("https://{}{}", parsed.authority, location))
}

fn parse_http_response(raw: &str) -> Result<HttpResponse, AuthError> {
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

    Ok(HttpResponse {
        status_code,
        headers,
        set_cookies,
        body: body.to_string(),
    })
}

fn extract_hidden_fields(html: &str) -> Result<(String, String), AuthError> {
    let lt = extract_hidden_input_value(html, "lt")
        .ok_or_else(|| AuthError::ParsingError("LT hidden field missing".to_string()))?;
    let execution = extract_hidden_input_value(html, "execution")
        .ok_or_else(|| AuthError::ParsingError("execution hidden field missing".to_string()))?;
    Ok((lt, execution))
}

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

fn parse_attribute(tag: &str, attribute: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let pattern = format!("{}={}", attribute, quote);
        if let Some(pos) = tag.find(&pattern) {
            let start = pos + pattern.len();
            let rest = tag.get(start..)?;
            let end = rest.find(quote)?;
            return rest.get(..end).map(|s| s.to_string());
        }
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
            self.responses
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Err(AuthError::CasServerError("missing mock response".to_string())))
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
                &[("location", "https://debsis.firat.edu.tr/login/index.php?ticket=ST-1")],
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
        let parsed = parse_http_response(raw).expect("response should parse");
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
        let err = resolve_redirect_url("https://jasig.firat.edu.tr/cas/login", "http://evil.local")
            .expect_err("insecure redirect must be rejected");
        assert!(matches!(err, AuthError::CasServerError(_)));
    }

    #[test]
    fn extract_hidden_input_value_supports_single_quote_and_reordered_attributes() {
        let html = r#"<input value='e1s1' type="hidden" name='execution'><input value="lt-9" name="lt">"#;
        let lt = extract_hidden_input_value(html, "lt").expect("lt should parse");
        let execution = extract_hidden_input_value(html, "execution").expect("execution should parse");
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
            &[("location", "https://jasig.firat.edu.tr/cas/login?service=abc")],
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
    fn logout_with_transport_returns_error_on_server_failure() {
        let adapter = CasAdapter::new(
            "https://jasig.firat.edu.tr/cas".to_string(),
            "https://debsis.firat.edu.tr".to_string(),
        );
        let transport = MockTransport::new(vec![Ok(response(500, &[], &[], ""))]);

        let err = adapter
            .logout_with_transport("cookie", &transport)
            .expect_err("logout on server error should fail");
        assert!(matches!(err, AuthError::CasServerError(_)));
    }
}
