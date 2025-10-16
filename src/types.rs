use crate::{request::RequestBody, response::ResponseWriter};
use async_trait::async_trait;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
    HEAD,
    TRACE,
}

#[derive(Eq, PartialEq)]
pub enum Mode {
    Http1,
    Http2,
}

pub type Handler = fn(&mut RequestBody, &mut ResponseWriter);

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn run(&self, req: &mut RequestBody, res: &mut ResponseWriter);
}

#[async_trait]
pub trait ErrorHandler: Send + Sync {
    async fn run(&self, msg: String, res: &mut ResponseWriter);
}
