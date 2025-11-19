use bytes::Bytes;
use futures_util::TryStreamExt;
use http_body_util::{BodyExt, BodyStream};
use hyper::header::HeaderName;
use hyper::{Request, Uri, Version, body::Incoming, header::HeaderValue};
use mime::Mime;
use multer::Multipart;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use url::form_urlencoded;
use uuid::Uuid;

use crate::types::{BoltError, FormData, FormFile};

#[allow(dead_code)]
pub struct RequestBody {
    pub inner: Option<Request<Incoming>>,
    pub raw_body: Option<Bytes>,
    params: HashMap<String, String>,
    form_data_result: Option<Result<FormData, Box<dyn std::error::Error + Send + Sync>>>,
    temp_paths: Vec<String>,
    socket: SocketAddr,
    pub extended: bool,
}

#[allow(dead_code)]
impl RequestBody {
    pub fn new(req: Request<Incoming>, socket: SocketAddr) -> Self {
        Self {
            inner: Some(req),
            params: HashMap::new(),
            form_data_result: None,
            temp_paths: Vec::new(),
            socket,
            extended: false,
            raw_body: None,
        }
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn remote_addr(&self) -> &SocketAddr {
        &self.socket
    }

    pub fn param(&self, key: &str) -> String {
        self.params.get(key).cloned().unwrap_or_default()
    }

    pub(crate) fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    pub fn method(&self) -> &hyper::Method {
        self.inner
            .as_ref()
            .expect("Cannot access method, request body was consumed.")
            .method()
    }

    pub fn path(&self) -> &str {
        self.inner
            .as_ref()
            .expect("Cannot access path, request body was consumed.")
            .uri()
            .path()
    }

    pub fn headers(&self) -> &hyper::HeaderMap {
        self.inner
            .as_ref()
            .expect("Cannot access headers, request body was consumed.")
            .headers()
    }

    pub fn set_headers(&mut self, key: &str, value: &str) {
        let key = HeaderName::from_bytes(key.as_bytes()).expect("Invalid header name");
        let value = HeaderValue::from_str(value).expect("Invalid header value");

        self.inner
            .as_mut()
            .expect("Cannot set headers, request body was consumed.")
            .headers_mut()
            .insert(key, value);
    }

    pub fn get_headers(&mut self, key: &str) -> Option<&HeaderValue> {
        self.inner
            .as_ref()
            .expect("Cannot access headers, request body was consumed.")
            .headers()
            .get(key)
    }

    pub fn uri(&self) -> &Uri {
        self.inner
            .as_ref()
            .expect("Cannot access uri, request body was consumed.")
            .uri()
    }

    pub fn version(&self) -> Version {
        self.inner
            .as_ref()
            .expect("Cannot access version, request body was consumed.")
            .version()
    }

    pub fn query(&self) -> HashMap<String, String> {
        self.inner
            .as_ref()
            .expect("Cannot access uri, request body was consumed.")
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

    pub async fn bytes(&mut self) -> Result<Bytes, hyper::Error> {
        if let Some(raw) = &self.raw_body {
            return Ok(raw.clone());
        }

        let req: Request<Incoming> = self
            .inner
            .take()
            .expect("Request body has already been consumed.");

        let (_, body) = req.into_parts();

        let collected = body.collect().await?;
        Ok(collected.to_bytes())
    }

    pub async fn text(&mut self) -> Result<String, BoltError> {
        let bytes = self.bytes().await?;
        let text = String::from_utf8(bytes.to_vec())?;
        Ok(text)
    }

    pub async fn json<T: DeserializeOwned>(&mut self) -> Result<T, BoltError> {
        let bytes = self.bytes().await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub async fn urlencoded(&mut self) -> Result<serde_json::Value, BoltError> {
        let bytes = self.bytes().await?;

        if self.extended {
            let structured: serde_json::Value = serde_urlencoded::from_bytes(&bytes)?;
            Ok(structured)
        } else {
            let s = String::from_utf8(bytes.to_vec())?;
            let mut map = HashMap::new();
            for (k, v) in form_urlencoded::parse(s.as_bytes()) {
                map.insert(k.into_owned(), v.into_owned());
            }
            Ok(serde_json::json!(map))
        }
    }

    pub fn get_cookie(&self, name: &str) -> Option<String> {
        self.inner
            .as_ref()
            .expect("Request body has already been consumed")
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

    pub async fn form_data(&mut self) -> Result<FormData, BoltError> {
        if let Some(Ok(fd)) = &self.form_data_result {
            return Ok(fd.clone());
        }
        if let Some(Err(e)) = &self.form_data_result {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )));
        }

        let header_opt = {
            let req_ref = self
                .inner
                .as_ref()
                .expect("Request body was consumed before form_data call.");
            req_ref.headers().get(hyper::header::CONTENT_TYPE).cloned()
        };

        let content_type = match header_opt {
            Some(header_value) => header_value.to_str()?.parse::<Mime>()?,
            None => {
                let err: BoltError = "Missing Content-Type header".into();
                self.form_data_result = Some(Err(err));
                return Err("Missing Content-Type header".into());
            }
        };

        if content_type.type_() != mime::MULTIPART || content_type.subtype() != mime::FORM_DATA {
            let err: BoltError = "Content-Type is not multipart/form-data".into();
            self.form_data_result = Some(Err(err));
            return Err("Content-Type is not multipart/form-data".into());
        }

        let boundary = content_type
            .get_param(mime::BOUNDARY)
            .ok_or("Missing boundary parameter in Content-Type")?
            .to_string();

        let (_, body) = self
            .inner
            .take()
            .expect("Request already consumed")
            .into_parts();

        let stream =
            BodyStream::new(body).try_filter_map(|frame| async move { Ok(frame.into_data().ok()) });

        let mut multipart = Multipart::new(stream, boundary);

        let mut form_data = FormData {
            files: Vec::new(),
            fields: HashMap::new(),
        };

        while let Ok(Some(mut field)) = multipart.next_field().await {
            let name = field.name().unwrap_or_default().to_string();

            if let Some(file_name) = field.file_name() {
                let filename = file_name.to_string();
                let unique_id = Uuid::new_v4();
                let temp_path =
                    std::env::temp_dir().join(format!("bolt_upload_{}_{}", unique_id, filename));

                let mut dest = tokio::fs::File::create(&temp_path).await?;

                while let Some(chunk) = field.chunk().await? {
                    dest.write_all(&chunk).await?;
                }

                self.temp_paths.push(temp_path.display().to_string());

                form_data.files.push(FormFile {
                    field_name: name,
                    file_name: filename,
                    content_type: field
                        .content_type()
                        .map(|m| m.essence_str().to_string())
                        .unwrap_or_default(),
                    temp_path: temp_path.display().to_string(),
                });
            } else {
                form_data.fields.insert(name, field.text().await?);
            }
        }

        self.form_data_result = Some(Ok(form_data.clone()));
        Ok(form_data)
    }

    pub async fn files(&mut self) -> Result<Vec<FormFile>, BoltError> {
        let form_data = self.form_data().await?;
        Ok(form_data.files)
    }

    pub async fn file(&mut self, name: &str) -> Result<Option<FormFile>, BoltError> {
        let files = self.files().await?;

        let file = files.iter().find(|f| f.field_name == name);
        Ok(file.cloned())
    }

    pub async fn cleanup(&mut self) {
        for path in self.temp_paths.drain(..) {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
}

impl Drop for RequestBody {
    fn drop(&mut self) {
        if self.temp_paths.is_empty() {
            return;
        }

        let paths = std::mem::take(&mut self.temp_paths);

        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::spawn(async move {
                for path in paths {
                    let _ = tokio::fs::remove_file(&path).await;
                }
            });
        } else {
            for path in paths {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}
