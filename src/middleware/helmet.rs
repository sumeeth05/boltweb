use async_trait::async_trait;

use crate::{request::RequestBody, response::ResponseWriter, types::Middleware};

pub struct Helmet;

#[async_trait]
impl Middleware for Helmet {
    async fn run(&self, _req: &mut RequestBody, res: &mut ResponseWriter) {
        res.set_header("X-DNS-Prefetch-Control", "off");
        res.set_header("X-Frame-Options", "SAMEORIGIN");
        res.set_header("X-Content-Type-Options", "nosniff");
        res.set_header("Referrer-Policy", "no-referrer");
        res.set_header("Permissions-Policy", "geolocation=(), microphone=()");
    }
}
