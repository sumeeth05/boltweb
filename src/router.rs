use radixmap::RadixMap;

use crate::types::{Handler, Method, Middleware};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
struct Node {
    pub handlers: HashMap<Method, Arc<dyn Handler>>,
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

    pub fn insert<H>(&mut self, path: &str, method: Method, handler: H)
    where
        H: Handler + 'static,
    {
        let key = path.as_bytes();

        if let Some(node) = self.router.get_mut(key) {
            node.handlers.insert(method, Arc::new(handler));
        } else {
            let mut node = Node::new();
            node.handlers.insert(method, Arc::new(handler));
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

    pub fn collect_middleware(&self, path: &str, method: Method) -> Vec<Arc<dyn Middleware>> {
        let mut entries = vec![];

        for (key_bytes, node) in self.router.iter() {
            let route = std::str::from_utf8(key_bytes).unwrap();

            if path.starts_with(route) {
                if let Some(mws) = node.middleware.get(&method) {
                    entries.push((route.to_string(), mws.clone()));
                }
            }
        }

        entries.sort_by_key(|(route, _)| route.len());

        let mut final_list = vec![];
        for (_, mws) in entries {
            for mw in mws {
                final_list.push(mw);
            }
        }

        final_list
    }

    pub fn match_path(&self, route: &str, uri: &str) -> Option<HashMap<String, String>> {
        let route_segments: Vec<&str> = route.trim_matches('/').split('/').collect();
        let uri_segments: Vec<&str> = uri.trim_matches('/').split('/').collect();

        let mut params = HashMap::new();

        for (i, route_seg) in route_segments.iter().enumerate() {
            if *route_seg == "*" {
                return Some(params);
            } else if route_seg.ends_with('*') {
                let key = route_seg
                    .trim_start_matches(':')
                    .trim_end_matches('*')
                    .to_string();

                let joined = uri_segments[i..].join("/");

                params.insert(key, joined);

                return Some(params);
            } else if let Some(uri_seg) = uri_segments.get(i) {
                if route_seg.starts_with(':') {
                    let key = route_seg.trim_start_matches(':').to_string();

                    params.insert(key, (*uri_seg).to_string());
                } else if route_seg != uri_seg {
                    return None;
                }
            } else {
                return None;
            }
        }

        if uri_segments.len() == route_segments.len() {
            Some(params)
        } else {
            None
        }
    }

    pub fn find(
        &self,
        path: &str,
        method: Method,
    ) -> Option<(&Arc<dyn Handler>, HashMap<String, String>)> {
        let mut best_match: Option<(&Arc<dyn Handler>, HashMap<String, String>, usize)> = None;

        for (key, node) in self.router.iter() {
            let route = std::str::from_utf8(key).unwrap();

            if let Some(params) = self.match_path(route, path) {
                if let Some(handler) = node.handlers.get(&method) {
                    let score = route
                        .split('/')
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            if s.starts_with(':') || s.ends_with('*') || s == "*" {
                                0
                            } else {
                                1
                            }
                        })
                        .sum();

                    if best_match.is_none() {
                        best_match = Some((handler, params, score));
                    } else {
                        if score > best_match.as_ref().unwrap().2 {
                            best_match = Some((handler, params, score));
                        }
                    }
                }
            }
        }

        best_match.map(|(handler, params, _)| (handler, params))
    }
}
