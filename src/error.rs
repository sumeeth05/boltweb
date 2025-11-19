use serde_json::json;

use crate::{error, response::ResponseWriter};

async fn default(message: String, res: &mut ResponseWriter) {
    let status = res.get_code(res.status);

    let msg = if status >= 500 {
        "Internal Server Error".to_string()
    } else {
        message
    };

    res.status(res.status)
        .json(&json!({"message": msg , "status" : status }));
}

error!(default);
