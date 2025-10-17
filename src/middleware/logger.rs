use crate::{
    bolt_middleware,
    {request::RequestBody, response::ResponseWriter},
};

pub async fn logger(req: &mut RequestBody, _res: &mut ResponseWriter) {
    println!("[{}] {}", req.method(), req.path());
}

bolt_middleware!(logger);
