use std::sync::Arc;

use async_trait::async_trait;

use crate::{request::RequestBody, response::ResponseWriter, types::Middleware};

pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allow_all: bool,
    pub allow_methods: String,
    pub allow_headers: String,
    pub allow_credentials: bool,
    pub max_age: Option<u32>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".into()],
            allow_all: true,
            allow_methods: "GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD".into(),
            allow_headers: "Content-Type, Authorization".into(),
            allow_credentials: false,
            max_age: Some(86400),
        }
    }
}

pub struct Cors {
    pub config: Arc<CorsConfig>,
}

#[async_trait]
impl Middleware for Cors {
    async fn run(&self, req: &mut RequestBody, res: &mut ResponseWriter) {
        let cfg = &self.config;

        res.set_header("Access-Control-Allow-Methods", &cfg.allow_methods)
            .set_header("Access-Control-Allow-Headers", &cfg.allow_headers);

        if cfg.allow_all {
            res.set_header("Access-Control-Allow-Origin", "*");
        } else if let Some(origin) = req.get_headers("Origin") {
            let origin_str = origin.to_str().unwrap_or("");
            if cfg.allowed_origins.contains(&origin_str.to_string()) {
                res.set_header("Access-Control-Allow-Origin", origin_str);
            }
        }

        if cfg.allow_credentials {
            res.set_header("Access-Control-Allow-Credentials", "true");
        }

        if let Some(max) = cfg.max_age {
            res.set_header("Access-Control-Max-Age", &max.to_string());
        }

        if *req.method() == hyper::Method::OPTIONS {
            res.status(204);
        }
    }
}
