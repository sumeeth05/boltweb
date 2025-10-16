use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get<U: DeserializeOwned>(
        &self,
        url: &str,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<U, reqwest::Error> {
        let mut req = self.client.get(url);
        if let Some(hdrs) = headers {
            for (k, v) in hdrs {
                req = req.header(k, v);
            }
        }
        let resp = req.send().await?;
        resp.json::<U>().await
    }

    pub async fn post<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<U, reqwest::Error> {
        let mut req = self.client.post(url).json(body);
        if let Some(hdrs) = headers {
            for (k, v) in hdrs {
                req = req.header(k, v);
            }
        }
        let resp = req.send().await?;
        resp.json::<U>().await
    }

    pub async fn put<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<U, reqwest::Error> {
        let mut req = self.client.put(url).json(body);
        if let Some(hdrs) = headers {
            for (k, v) in hdrs {
                req = req.header(k, v);
            }
        }
        let resp = req.send().await?;
        resp.json::<U>().await
    }

    pub async fn patch<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<U, reqwest::Error> {
        let mut req = self.client.patch(url).json(body);
        if let Some(hdrs) = headers {
            for (k, v) in hdrs {
                req = req.header(k, v);
            }
        }
        let resp = req.send().await?;
        resp.json::<U>().await
    }

    pub async fn delete<U: DeserializeOwned>(
        &self,
        url: &str,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<U, reqwest::Error> {
        let mut req = self.client.delete(url);
        if let Some(hdrs) = headers {
            for (k, v) in hdrs {
                req = req.header(k, v);
            }
        }
        let resp = req.send().await?;
        resp.json::<U>().await
    }
}
