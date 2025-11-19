use base64::{Engine, engine::general_purpose};
use bytes::Bytes;
use cookie::{Cookie, SameSite};
use http_body_util::Full;
use hyper::{
    HeaderMap, Response,
    header::{HeaderName, HeaderValue},
};
use mime_guess::from_path;
use serde::Serialize;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;

use crate::http::StatusCode;

pub struct ResponseWriter {
    pub body: String,
    pub headers: HeaderMap,
    pub status: StatusCode,
    pub has_error: bool,
}

#[allow(dead_code)]
impl ResponseWriter {
    pub fn new() -> Self {
        Self {
            body: "".into(),
            headers: HeaderMap::new(),
            status: StatusCode::OK,
            has_error: false,
        }
    }

    pub fn status(&mut self, status: StatusCode) -> &mut Self {
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
                self.status = StatusCode::InternalServerError;
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
                    self.error(
                        StatusCode::InternalServerError,
                        &format!("Failed to read file: {}", e),
                    );
                    return;
                }

                let mime_type = from_path(path_ref).first_or_octet_stream().to_string();

                self.status(StatusCode::OK)
                    .set_header("Content-Type", &mime_type)
                    .bytes(&buf);
            }
            Err(_) => {
                self.error(StatusCode::NotFound, "File not found");
            }
        }
    }

    pub fn bytes(&mut self, bytes: &[u8]) -> &mut Self {
        let encoded = general_purpose::STANDARD.encode(bytes);
        self.body = encoded;
        self.set_header("Content-Type", "application/octet-stream");
        self
    }

    pub fn get_code(&self, code: StatusCode) -> u16 {
        match code {
            StatusCode::Continue => return 100,
            StatusCode::SwitchingProtocols => return 101,
            StatusCode::Processing => return 102,
            StatusCode::EarlyHints => return 103,
            StatusCode::OK => return 200,
            StatusCode::Created => return 201,
            StatusCode::Accepted => return 202,
            StatusCode::NonAuthoritativeInformation => return 203,
            StatusCode::NoContent => return 204,
            StatusCode::ResetContent => return 205,
            StatusCode::PartialContent => return 206,
            StatusCode::MovedPermanently => return 301,
            StatusCode::Found => return 302,
            StatusCode::SeeOther => return 303,
            StatusCode::NotModified => return 304,
            StatusCode::TemporaryRedirect => return 307,
            StatusCode::PermanentRedirect => return 308,
            StatusCode::BadRequest => return 400,
            StatusCode::Unauthorized => return 401,
            StatusCode::PaymentRequired => return 402,
            StatusCode::Forbidden => return 403,
            StatusCode::NotFound => return 404,
            StatusCode::MethodNotAllowed => return 405,
            StatusCode::NotAcceptable => return 406,
            StatusCode::ProxyAuthenticationRequired => return 407,
            StatusCode::RequestTimeout => return 408,
            StatusCode::Conflict => return 409,
            StatusCode::Gone => return 410,
            StatusCode::LengthRequired => return 411,
            StatusCode::PreconditionFailed => return 412,
            StatusCode::ContentTooLarge => return 413,
            StatusCode::URITooLong => return 414,
            StatusCode::UnsupportedMediaType => return 415,
            StatusCode::TooManyRequests => return 429,
            StatusCode::InternalServerError => return 500,
            StatusCode::NotImplemented => return 501,
            StatusCode::BadGateway => return 502,
            StatusCode::ServiceUnavailable => return 503,
            StatusCode::GatewayTimeout => return 504,
            StatusCode::HTTPVersionNotSupported => return 505,
        }
    }

    pub fn error(&mut self, status: StatusCode, msg: &str) -> &mut Self {
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
        let mut cookie_builder = Cookie::build((name, value))
            .path(path.unwrap_or("/"))
            .secure(secure)
            .http_only(http_only);

        if let Some(d) = domain {
            cookie_builder = cookie_builder.domain(d);
        }

        if let Some(age) = max_age {
            cookie_builder = cookie_builder.max_age(time::Duration::seconds(age));
        }

        if let Some(ss) = same_site {
            match ss.to_lowercase().as_str() {
                "lax" => cookie_builder = cookie_builder.same_site(SameSite::Lax),
                "strict" => cookie_builder = cookie_builder.same_site(SameSite::Strict),
                "none" => cookie_builder = cookie_builder.same_site(SameSite::None).secure(true),
                _ => {}
            }
        }

        self.headers.append(
            hyper::header::SET_COOKIE,
            hyper::header::HeaderValue::from_str(&cookie_builder.to_string()).unwrap(),
        );

        self
    }

    pub fn into_response(&self) -> Response<Full<Bytes>> {
        let status = &self.status;

        let status_code = self.get_code(status.clone());
        let body = &self.body;
        let mut builder = Response::builder().status(status_code);

        for (key, value) in self.headers.iter() {
            builder = builder.header(key, value);
        }

        builder
            .body(Full::new(Bytes::from(body.to_owned())))
            .unwrap()
    }

    pub fn strip_header(&mut self, key: &str) {
        if let Ok(key_name) = hyper::header::HeaderName::from_bytes(key.as_bytes()) {
            self.headers.remove(key_name);
        }
    }
}
