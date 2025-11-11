use serde_json::json;

use crate::{bolt_error_handler, response::ResponseWriter};

async fn default(msg: String, res: &mut ResponseWriter) {
    let status = res.status;
    res.status(status)
        .json(&json!({ "error": msg, "status" : status }));
}

bolt_error_handler!(default);
