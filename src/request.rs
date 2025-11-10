use std::collections::HashMap;

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::header::HeaderName;
use hyper::{Request, Uri, Version, body::Incoming, header::HeaderValue};
use serde::de::DeserializeOwned;
use url::form_urlencoded;

pub struct RequestBody {
    inner: Request<Incoming>,
    params: HashMap<String, String>,
}

#[allow(dead_code)]
impl RequestBody {
    pub fn new(req: Request<Incoming>) -> Self {
        Self {
            inner: req,
            params: HashMap::new(),
        }
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub(crate) fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    pub fn method(&self) -> &hyper::Method {
        self.inner.method()
    }

    pub fn path(&self) -> &str {
        self.inner.uri().path()
    }

    pub fn headers(&self) -> &hyper::HeaderMap {
        self.inner.headers()
    }

    pub fn set_headers(&mut self, key: &str, value: &str) {
        let key = HeaderName::from_bytes(key.as_bytes()).expect("Invalid header name");
        let value = HeaderValue::from_str(value).expect("Invalid header value");

        self.inner.headers_mut().insert(key, value);
    }

    pub fn get_headers(&mut self, key: &str) -> Option<&HeaderValue> {
        self.inner.headers().get(key)
    }

    pub fn uri(&self) -> &Uri {
        self.inner.uri()
    }

    pub fn version(&self) -> Version {
        self.inner.version()
    }

    pub fn query(&self) -> HashMap<String, String> {
        self.inner
            .uri()
            .query()
            .map(|q| {
                form_urlencoded::parse(q.as_bytes())
                    .into_owned()
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default()
    }

    pub fn query_param(&self, key: &str) -> Option<String> {
        let query_params = self.query();
        query_params.get(key).cloned()
    }

    pub async fn bytes(self) -> Result<Bytes, hyper::Error> {
        let (_, body) = self.inner.into_parts();
        let collected = body.collect().await?;
        Ok(collected.to_bytes())
    }

    pub async fn text(self) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = self.bytes().await?;
        let text = String::from_utf8(bytes.to_vec())?;
        Ok(text)
    }

    pub async fn json<T: DeserializeOwned>(self) -> Result<T, Box<dyn std::error::Error>> {
        let bytes = self.bytes().await?;
        let value = serde_json::from_slice(&bytes)?;
        Ok(value)
    }

    pub fn get_cookie(&self, name: &str) -> Option<String> {
        self.inner
            .headers()
            .get(hyper::header::COOKIE)?
            .to_str()
            .ok()
            .and_then(|cookie_header| {
                cookie_header.split(';').map(|s| s.trim()).find_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next()?;
                    let value = parts.next()?;
                    if key == name {
                        Some(value.to_string())
                    } else {
                        None
                    }
                })
            })
    }

}
