//! Router and Request Handler

use crate::http::{Request, Response, Method};
use std::sync::RwLock;
use std::collections::HashMap;

type Handler = fn(&Request) -> Response;

pub struct Router {
    /// Exact match routes (O(1) lookup)
    get_routes: RwLock<HashMap<String, Handler>>,
    post_routes: RwLock<HashMap<String, Handler>>,
    /// Wildcard routes (prefix match) - separate for faster scanning
    get_wildcards: RwLock<Vec<(String, Handler)>>,
    post_wildcards: RwLock<Vec<(String, Handler)>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            get_routes: RwLock::new(HashMap::new()),
            post_routes: RwLock::new(HashMap::new()),
            get_wildcards: RwLock::new(Vec::new()),
            post_wildcards: RwLock::new(Vec::new()),
        }
    }

    pub fn get(&self, path: &str, handler: Handler) {
        if path.ends_with('*') {
            // Store prefix without the * for faster matching
            let prefix = path.trim_end_matches('*').to_string();
            self.get_wildcards.write().unwrap().push((prefix, handler));
        } else {
            self.get_routes.write().unwrap().insert(path.to_string(), handler);
        }
    }

    pub fn post(&self, path: &str, handler: Handler) {
        if path.ends_with('*') {
            let prefix = path.trim_end_matches('*').to_string();
            self.post_wildcards.write().unwrap().push((prefix, handler));
        } else {
            self.post_routes.write().unwrap().insert(path.to_string(), handler);
        }
    }

    pub fn handle(&self, request: &Request) -> Response {
        let (routes, wildcards) = match request.method {
            Method::GET => (&self.get_routes, &self.get_wildcards),
            Method::POST => (&self.post_routes, &self.post_wildcards),
            _ => (&self.get_routes, &self.get_wildcards),
        };

        // Exact match (O(1))
        if let Some(handler) = routes.read().unwrap().get(&request.path) {
            return handler(request);
        }

        // Wildcard match - only scan wildcard routes, not all routes
        let wildcards = wildcards.read().unwrap();
        for (prefix, handler) in wildcards.iter() {
            if request.path.starts_with(prefix) {
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
}
