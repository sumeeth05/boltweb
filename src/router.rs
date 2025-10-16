use radixmap::RadixMap;

use crate::types::{Handler, Method, Middleware};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
struct Node {
    handlers: HashMap<Method, Handler>,
    middleware: HashMap<Method, Vec<Arc<dyn Middleware>>>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            middleware: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct Router {
    router: RadixMap<Node>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            router: RadixMap::new(),
        }
    }

    pub fn insert(&mut self, path: &str, method: Method, handler: Handler) {
        let key = path.as_bytes();

        if let Some(node) = self.router.get_mut(key) {
            node.handlers.insert(method, handler);
        } else {
            let mut node = Node::new();
            node.handlers.insert(method, handler);
            let _ = self.router.insert(key.to_vec(), node);
        }
    }

    pub fn insert_middleware(&mut self, path: &str, method: Method, mw: Arc<dyn Middleware>) {
        let key = path.as_bytes();

        if let Some(node) = self.router.get_mut(key) {
            node.middleware.entry(method).or_default().push(mw);
        } else {
            let mut node = Node::new();
            node.middleware.insert(method, vec![mw]);
            let _ = self.router.insert(key.to_vec(), node);
        }
    }

    pub fn find(&self, path: &str, method: Method) -> Option<(&Handler, HashMap<String, String>)> {
        for (key, node) in self.router.iter() {
            let route = std::str::from_utf8(key).unwrap();
            if let Some(params) = self.match_path(route, path) {
                if let Some(handler) = node.handlers.get(&method) {
                    return Some((handler, params));
                }
            }
        }
        None
    }

    pub fn match_path<'a>(&self, route: &str, uri: &str) -> Option<HashMap<String, String>> {
        let route_segments: Vec<&str> = route.trim_matches('/').split('/').collect();
        let uri_segments: Vec<&str> = uri.trim_matches('/').split('/').collect();

        if route_segments.len() != uri_segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (route, uri) in route_segments.iter().zip(uri_segments.iter()) {
            if route.starts_with(':') {
                let key = route.trim_start_matches(':').to_string();
                params.insert(key, uri.to_string());
            } else if route != uri {
                return None;
            }
        }

        Some(params)
    }

    pub fn collect_middleware(&self, path: &str, method: Method) -> Vec<Arc<dyn Middleware>> {
        let mut result = Vec::new();

        for (key_bytes, node) in self.router.iter() {
            let route = std::str::from_utf8(key_bytes).unwrap();

            if path.starts_with(route) {
                if let Some(mws) = node.middleware.get(&method) {
                    result.extend(mws.iter());
                }
            }
        }

        result.into_iter().cloned().collect()
    }
}
