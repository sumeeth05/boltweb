use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request};

use hyper::http::request::Builder;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{Client as HyperClient, connect::HttpConnector};
use hyper_util::rt::TokioExecutor;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::types::BoltError;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>, Full<Bytes>>,
}

#[allow(dead_code)]
impl Client {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let client = HyperClient::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);
        Self { client }
    }

    fn apply_headers(mut builder: Builder, headers: &Option<Value>) -> Builder {
        if let Some(Value::Object(map)) = headers {
            for (k, v) in map {
                if let Some(s) = v.as_str() {
                    builder = builder.header(k, s);
                }
            }
        }
        builder
    }

    pub async fn fetch(&self, url: &str, headers: &Option<Value>) -> Result<String, BoltError> {
        let mut builder = Request::builder().method(Method::GET).uri(url);
        builder = Self::apply_headers(builder, &headers);

        let req = builder.body(Full::new(Bytes::new()))?;
        let resp = self.client.request(req).await?;
        let body = resp.into_body().collect().await?.to_bytes();

        Ok(String::from_utf8_lossy(&body).to_string())
    }

    async fn send_json<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: &T,
        headers: &Option<Value>,
    ) -> Result<U, BoltError> {
        let body_bytes = serde_json::to_vec(body)?;

        let mut builder = Request::builder()
            .method(method)
            .uri(url)
            .header("Content-Type", "application/json");

        builder = Self::apply_headers(builder, &headers);

        let req = builder.body(Full::new(Bytes::from(body_bytes)))?;
        let resp = self.client.request(req).await?;
        let bytes = resp.into_body().collect().await?.to_bytes();

        Ok(serde_json::from_slice(&bytes)?)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        headers: &Option<Value>,
    ) -> Result<T, BoltError> {
        let mut builder = Request::builder().method(Method::GET).uri(url);
        builder = Self::apply_headers(builder, &headers);

        let req = builder.body(Full::new(Bytes::new()))?;
        let resp = self.client.request(req).await?;
        let body = resp.into_body().collect().await?.to_bytes();

        Ok(serde_json::from_slice(&body)?)
    }

    pub async fn post<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
        headers: &Option<Value>,
    ) -> Result<U, BoltError> {
        self.send_json(Method::POST, url, body, headers).await
    }

    pub async fn put<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
        headers: &Option<Value>,
    ) -> Result<U, BoltError> {
        self.send_json(Method::PUT, url, body, headers).await
    }

    pub async fn patch<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
        headers: &Option<Value>,
    ) -> Result<U, BoltError> {
        self.send_json(Method::PATCH, url, body, headers).await
    }

    pub async fn delete<U: DeserializeOwned>(
        &self,
        url: &str,
        headers: &Option<Value>,
    ) -> Result<U, BoltError> {
        let mut builder = Request::builder().method(Method::DELETE).uri(url);
        builder = Self::apply_headers(builder, &headers);

        let req = builder.body(Full::new(Bytes::new()))?;

        let resp = self.client.request(req).await?;

        let body_bytes = resp.into_body().collect().await?.to_bytes();

        Ok(serde_json::from_slice(&body_bytes)?)
    }

    pub async fn delete_with_payload<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
        headers: &Option<Value>,
    ) -> Result<U, BoltError> {
        self.send_json(Method::DELETE, url, body, headers).await
    }
}
