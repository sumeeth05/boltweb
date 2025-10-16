use async_trait::async_trait;

use crate::{request::RequestBody, response::ResponseWriter, types::Middleware};

pub struct Logger;

#[async_trait]
impl Middleware for Logger {
    async fn run(&self, req: &mut RequestBody, _res: &mut ResponseWriter) {
        println!("[{}] {}", req.method(), req.path());
    }
}
