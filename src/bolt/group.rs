use std::sync::Arc;

use crate::bolt::{
    Bolt,
    types::{Handler, Method, Middleware},
};

pub struct Group<'a> {
    pub prefix: String,
    pub app: &'a mut Bolt,
}

impl<'a> Group<'a> {
    pub fn get(&mut self, path: &str, handler: Handler) {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::GET, &full_path, handler);
    }

    pub fn post(&mut self, path: &str, handler: Handler) {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::POST, &full_path, handler);
    }

    pub fn put(&mut self, path: &str, handler: Handler) {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::PUT, &full_path, handler);
    }

    pub fn patch(&mut self, path: &str, handler: Handler) {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::PATCH, &full_path, handler);
    }

    pub fn delete(&mut self, path: &str, handler: Handler) {
        let full_path: String = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::DELETE, &full_path, handler);
    }

    pub fn middleware(&mut self, path: &str, method: Option<Method>, mw: Arc<dyn Middleware>) {
        use crate::bolt::types::Method::*;

        let full_path: String = format!("{}{}", self.prefix, path);

        match method {
            Some(m) => self.app.router.insert_middleware(&full_path, m, mw.clone()),
            None => {
                for m in [GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD, TRACE] {
                    self.app.router.insert_middleware(&full_path, m, mw.clone());
                }
            }
        }
    }
}
