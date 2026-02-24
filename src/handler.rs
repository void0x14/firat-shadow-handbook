//! Router and Request Handler

use crate::http::{Request, Response, Method};
use std::sync::RwLock;
use std::collections::HashMap;

type Handler = fn(&Request) -> Response;

pub struct Router {
    get_routes: RwLock<HashMap<String, Handler>>,
    post_routes: RwLock<HashMap<String, Handler>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            get_routes: RwLock::new(HashMap::new()),
            post_routes: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, path: &str, handler: Handler) {
        self.get_routes.write().unwrap().insert(path.to_string(), handler);
    }

    pub fn post(&self, path: &str, handler: Handler) {
        self.post_routes.write().unwrap().insert(path.to_string(), handler);
    }

    pub fn handle(&self, request: &Request) -> Response {
        let routes = match request.method {
            Method::GET => &self.get_routes,
            Method::POST => &self.post_routes,
            _ => &self.get_routes,
        };

        let routes = routes.read().unwrap();

        // Exact match
        if let Some(handler) = routes.get(&request.path) {
            return handler(request);
        }

        // Pattern matching for dynamic routes
        for (pattern, handler) in routes.iter() {
            if self.match_pattern(pattern, &request.path) {
                return handler(request);
            }
        }

        // 404 Not Found
        Response::html(404, r#"
<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <title>404 - Sayfa Bulunamadı</title>
    <style>
        body { font-family: system-ui; display: flex; justify-content: center; align-items: center; min-height: 100vh; margin: 0; background: #1a1a1a; color: #fff; }
        .error { text-align: center; }
        h1 { font-size: 6rem; margin: 0; color: #ff4444; }
        p { color: #888; }
        a { color: #4a9eff; text-decoration: none; }
    </style>
</head>
<body>
    <div class="error">
        <h1>404</h1>
        <p>Sayfa bulunamadı</p>
        <a href="/">Ana Sayfaya Dön</a>
    </div>
</body>
</html>
"#)
    }

    fn match_pattern(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        for (p, actual) in pattern_parts.iter().zip(path_parts.iter()) {
            if p.starts_with(':') {
                continue; // Dynamic segment
            }
            if p != actual {
                return false;
            }
        }

        true
    }
}
