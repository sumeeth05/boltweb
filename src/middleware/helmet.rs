use crate::{bolt_middleware, request::RequestBody, response::ResponseWriter};

pub async fn helmet(_req: &mut RequestBody, res: &mut ResponseWriter) {
    res.set_header("X-DNS-Prefetch-Control", "off");
    res.set_header("X-Frame-Options", "SAMEORIGIN");
    res.set_header("X-Content-Type-Options", "nosniff");
    res.set_header("Referrer-Policy", "no-referrer");
    res.set_header("Permissions-Policy", "geolocation=(), microphone=()");
}

bolt_middleware!(helmet);
