use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request};

use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{Client as HyperClient, connect::HttpConnector};
use hyper_util::rt::TokioExecutor;
use serde::Serialize;
use serde::de::DeserializeOwned;

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

    pub async fn fetch(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Build the GET request
        let req = Request::builder()
            .method(Method::GET)
            .uri(url)
            .body(Full::new(Bytes::new()))?;

        let resp = self.client.request(req).await?;

        let body_bytes = resp.into_body().collect().await?.to_bytes();

        Ok(String::from_utf8_lossy(&body_bytes).to_string())
    }

    async fn send_json<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: &T,
    ) -> Result<U, Box<dyn std::error::Error + Send + Sync>> {
        let body_bytes = serde_json::to_vec(body)?;
        let req = Request::builder()
            .method(method)
            .uri(url)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(body_bytes)))?;
        let resp = self.client.request(req).await?;
        let body_bytes = resp.into_body().collect().await?.to_bytes();
        Ok(serde_json::from_slice(&body_bytes)?)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let req = Request::builder()
            .method(Method::GET)
            .uri(url)
            .body(Full::new(Bytes::new()))?;

        let resp = self.client.request(req).await?;

        let body_bytes = resp.into_body().collect().await?.to_bytes();

        let parsed: T = serde_json::from_slice(&body_bytes)?;
        Ok(parsed)
    }

    pub async fn post<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<U, Box<dyn std::error::Error + Send + Sync>> {
        self.send_json(Method::POST, url, body).await
    }

    pub async fn put<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<U, Box<dyn std::error::Error + Send + Sync>> {
        self.send_json(Method::PUT, url, body).await
    }

    pub async fn patch<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<U, Box<dyn std::error::Error + Send + Sync>> {
        self.send_json(Method::PATCH, url, body).await
    }

    pub async fn delete<U: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<U, Box<dyn std::error::Error + Send + Sync>> {
        let req = Request::builder()
            .method(Method::DELETE)
            .uri(url)
            .body(Full::new(Bytes::new()))?;

        let resp = self.client.request(req).await?;

        let body_bytes = resp.into_body().collect().await?.to_bytes();

        Ok(serde_json::from_slice(&body_bytes)?)
    }

    pub async fn delete_with_payload<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<U, Box<dyn std::error::Error + Send + Sync>> {
        self.send_json(Method::DELETE, url, body).await
    }
}
