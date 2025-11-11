use crate::{bolt_middleware, request::RequestBody, response::ResponseWriter};

pub async fn logger(req: &mut RequestBody, _res: &mut ResponseWriter) {
    req.log = true;
}

bolt_middleware!(logger);
