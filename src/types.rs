use crate::{request::RequestBody, response::ResponseWriter};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error as StdError;

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
    async fn run(&self, req: &mut RequestBody, res: &mut ResponseWriter);
}

pub type BoltError = Box<dyn StdError + Send + Sync>;

#[allow(dead_code)]
pub type BoltResult<T> = Result<T, BoltError>;
