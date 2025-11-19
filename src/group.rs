use std::sync::Arc;

use crate::{
    App,
    types::{Handler, Method, Middleware},
};

#[allow(dead_code)]
pub struct Group<'a> {
    pub prefix: String,
    pub app: &'a mut App,
}

#[allow(dead_code)]
impl<'a> Group<'a> {
    pub fn get<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::GET, &full_path, handler);
    }

    pub fn post<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::POST, &full_path, handler);
    }

    pub fn put<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::PUT, &full_path, handler);
    }

    pub fn patch<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::PATCH, &full_path, handler);
    }

    pub fn delete<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        let full_path = format!("{}{}", self.prefix, path);
        self.app.add_route(Method::DELETE, &full_path, handler);
    }

    pub fn middleware(&mut self, path: &str, method: Option<Method>, mw: Arc<dyn Middleware>) {
        use crate::types::Method::*;

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

    pub fn group(&'a mut self, path: &str) -> Group<'a> {
        let base = self.prefix.trim_end_matches('/');

        let child_path = path.trim_start_matches('/');

        let new_prefix = format!("{}/{}", base, child_path);
        Group {
            prefix: new_prefix,
            app: self.app,
        }
    }
}
