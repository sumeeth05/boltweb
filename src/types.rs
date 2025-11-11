use std::collections::HashMap;

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
#[allow(dead_code)]
pub enum Mode {
    Http1,
    Http2,
}

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn run(&self, req: &mut RequestBody, res: &mut ResponseWriter);
}

#[async_trait]
pub trait ErrorHandler: Send + Sync {
    async fn run(&self, msg: String, res: &mut ResponseWriter);
}

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, req: &mut RequestBody, res: &mut ResponseWriter);
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FormFile {
    pub field_name: String,
    pub file_name: String,
    pub content_type: String,
    pub temp_path: String,
}

#[derive(Debug, Clone)]
pub struct FormData {
    pub files: Vec<FormFile>,
    pub fields: HashMap<String, String>,
}

#[allow(dead_code)]
pub type BoltError = Box<dyn std::error::Error + Send + Sync>;

#[allow(dead_code)]
pub type BoltResult<T> = Result<T, BoltError>;
