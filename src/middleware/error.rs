use serde_json::json;

use crate::{bolt_error_handler, response::ResponseWriter};

async fn default(msg: String, res: &mut ResponseWriter) {
    res.status(res.status).json(&json!({ "error": msg }));
}

bolt_error_handler!(default);
