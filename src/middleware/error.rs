use async_trait::async_trait;
use serde_json::json;

use crate::{response::ResponseWriter, types::ErrorHandler};

pub struct DefaultErrorHandler;

#[async_trait]
impl ErrorHandler for DefaultErrorHandler {
    async fn run(&self, msg: String, res: &mut ResponseWriter) {
        res.status(res.status).json(&json!({
            "error": msg
        }));
    }
}
