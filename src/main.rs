//! Fırat Shadow Handbook - Zero Dependency HTTP Server
//! 
//! Pure Rust std::net implementation - no external crates

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::fs;
use std::path::Path;

mod http;
mod handler;
mod config;

use http::{Request, Response, Method};
use handler::Router;

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

    while RUNNING.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, addr)) => {
                let router = Arc::clone(&router);
                thread::spawn(move || {
                    handle_connection(stream, addr, &router);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream, addr: std::net::SocketAddr, router: &Router) {
    let reader = BufReader::new(&stream);
    
    let request = match parse_request(reader) {
        Some(r) => r,
        None => return,
    };

    println!("[{}] {} {} {:?}", addr.ip(), request.method, request.path, request.headers.get("User-Agent"));

    let response = router.handle(&request);
    send_response(&mut stream, &response);
}

fn parse_request<R: BufRead>(reader: R) -> Option<Request> {
    let mut lines = reader.lines();
    
    let first_line = lines.next()?.ok()?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    if parts.len() < 3 {
        return None;
    }

    let method = match parts[0] {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        _ => Method::GET,
    };

    let path = parts[1].to_string();
    
    let mut headers = std::collections::HashMap::new();
    let mut body = String::new();

    for line in lines {
        match line {
            Ok(l) if l.is_empty() => break,
            Ok(l) => {
                if let Some((key, value)) = l.split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
            Err(_) => break,
        }
    }

    // Read body if Content-Length exists
    if let Some(len) = headers.get("Content-Length") {
        if let Ok(content_len) = len.parse::<usize>() {
            // Note: BufReader consumed lines, need different approach for body
            // For now, skip body parsing in this simplified version
            let _ = content_len;
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
    let status_text = match response.status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let headers: String = response.headers
        .iter()
        .map(|(k, v)| format!("{}: {}\r\n", k, v))
        .collect();

    let response_text = format!(
        "HTTP/1.1 {} {}\r\n{}\r\n{}",
        response.status, status_text, headers, response.body
    );

    let _ = stream.write_all(response_text.as_bytes());
    let _ = stream.flush();
}

fn setup_routes(router: &Router) {
    // Static files - relative to src folder
    router.get("/", |req| serve_file("../web/index.html", "text/html"));
    router.get("/css/:file", |req| {
        let file = req.path.strip_prefix("/css/").unwrap_or("");
        serve_file(&format!("../web/css/{}", file), "text/css")
    });
    router.get("/js/:file", |req| {
        let file = req.path.strip_prefix("/js/").unwrap_or("");
        serve_file(&format!("../web/js/{}", file), "application/javascript")
    });
    router.get("/i18n/:file", |req| {
        let file = req.path.strip_prefix("/i18n/").unwrap_or("");
        serve_file(&format!("../web/i18n/{}", file), "application/json")
    });
    
    // API endpoints
    router.get("/api/health", |_| Response::json(200, r#"{"status":"ok"}"#));
    router.get("/api/config", |_| Response::json(200, r#"{"version":"0.1.0"}"#));
}

fn serve_file(path: &str, content_type: &str) -> Response {
    match fs::read_to_string(path) {
        Ok(content) => Response {
            status: 200,
            headers: vec![
                ("Content-Type".to_string(), content_type.to_string()),
                ("Content-Length".to_string(), content.len().to_string()),
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
