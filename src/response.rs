use base64::{Engine, engine::general_purpose};
use bytes::Bytes;
use http_body_util::Full;
use hyper::{
    HeaderMap, Response,
    header::{HeaderName, HeaderValue},
};
use mime_guess::from_path;
use serde::Serialize;
use std::path::Path;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc2822};
use tokio::fs;
use tokio::io::AsyncReadExt;

pub struct ResponseWriter {
    pub body: String,
    pub headers: HeaderMap,
    pub status: u16,
    pub has_error: bool,
}

#[allow(dead_code)]
impl ResponseWriter {
    pub fn new() -> Self {
        Self {
            body: "".into(),
            headers: HeaderMap::new(),
            status: 200,
            has_error: false,
        }
    }

    pub fn status(&mut self, status: u16) -> &mut Self {
        self.status = status;
        self
    }

    pub fn set_header(&mut self, key: &str, value: &str) -> &mut Self {
        self.headers.insert(
            HeaderName::from_bytes(key.as_bytes()).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
        self
    }

    pub fn get_header(&self, key: &str) -> Option<&HeaderValue> {
        self.headers.get(key)
    }

    pub fn send(&mut self, body: &str) -> &mut Self {
        self.body = body.into();
        self
    }

    pub fn json<T: Serialize>(&mut self, data: &T) -> &mut Self {
        match serde_json::to_string(data) {
            Ok(body) => {
                self.set_header("Content-Type", "application/json");
                self.body = body;
            }
            Err(_) => {
                self.set_header("Content-Type", "application/json");
                self.body = r#"{"error":"Failed to serialize JSON"}"#.to_string();
                self.status = 500;
            }
        }
        self
    }

    pub fn html(&mut self, html: &str) -> &mut Self {
        self.set_header("Content-Type", "text/html; charset=utf-8");
        self.body = html.to_string();
        self
    }

    pub async fn file<P: AsRef<Path>>(&mut self, path: P) {
        let path_ref = path.as_ref();

        match fs::File::open(path_ref).await {
            Ok(mut file) => {
                let mut buf = Vec::new();
                if let Err(e) = file.read_to_end(&mut buf).await {
                    self.error(500, &format!("Failed to read file: {}", e));
                    return;
                }

                let mime_type = from_path(path_ref).first_or_octet_stream().to_string();

                self.status(200)
                    .set_header("Content-Type", &mime_type)
                    .bytes(&buf);
            }
            Err(_) => {
                self.error(404, "File not found");
            }
        }
    }

    pub fn bytes(&mut self, bytes: &[u8]) -> &mut Self {
        let encoded = general_purpose::STANDARD.encode(bytes);
        self.body = encoded;
        self.set_header("Content-Type", "application/octet-stream");
        self
    }

    pub fn error(&mut self, status: u16, msg: &str) -> &mut Self {
        self.status = status;
        self.body = msg.to_string();
        self.has_error = true;
        self
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }

    pub fn cookie(
        &mut self,
        name: &str,
        value: &str,
        max_age: Option<i64>,
        path: Option<&str>,
        domain: Option<&str>,
        secure: bool,
        http_only: bool,
        same_site: Option<&str>,
    ) -> &mut Self {
        let mut cookie = format!("{}={}", name, value);

        if let Some(age) = max_age {
            if let Ok(expires) =
                (OffsetDateTime::now_utc() + Duration::seconds(age)).format(&Rfc2822)
            {
                cookie.push_str(&format!("; Max-Age={}; Expires={}", age, expires));
            }
        }

        if let Some(p) = path {
            cookie.push_str(&format!("; Path={}", p));
        }

        if let Some(d) = domain {
            cookie.push_str(&format!("; Domain={}", d));
        }

        if secure {
            cookie.push_str("; Secure");
        }

        if http_only {
            cookie.push_str("; HttpOnly");
        }

        if let Some(same_site_val) = same_site {
            cookie.push_str(&format!("; SameSite={}", same_site_val));
        }

        self.headers.append(
            hyper::header::SET_COOKIE,
            hyper::header::HeaderValue::from_str(&cookie).unwrap(),
        );

        self
    }

    pub fn into_response(self) -> Response<Full<Bytes>> {
        let mut builder = Response::builder().status(self.status);

        for (key, value) in self.headers.iter() {
            builder = builder.header(key, value);
        }

        builder.body(Full::new(Bytes::from(self.body))).unwrap()
    }

    pub fn strip_header(&mut self, key: &str) {
        if let Ok(key_name) = hyper::header::HeaderName::from_bytes(key.as_bytes()) {
            self.headers.remove(key_name);
        }
    }
}
