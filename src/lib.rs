use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{
    Request,
    body::Incoming,
    server::conn::{http1, http2},
    service::service_fn,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;

use crate::{
    client::Client,
    group::Group,
    middleware::error::DefaultErrorHandler,
    request::RequestBody,
    response::ResponseWriter,
    router::Router,
    types::{ErrorHandler, Handler, Method, Middleware, Mode},
};

pub mod client;
mod group;
pub mod middleware;
pub mod request;
pub mod response;
mod router;
pub mod types;

pub struct Bolt {
    router: Router,
    error_handler: Arc<dyn ErrorHandler>,
    client: Client,
}

impl Bolt {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            error_handler: Arc::new(DefaultErrorHandler),
            client: Client::new(),
        }
    }

    pub async fn run(
        &self,
        addr: &str,
        mode: Mode,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("⚡ A high performance & minimalist web framework in rust.");
        println!(
            r#"
    __          ____
   / /_  ____  / / /_
  / __ \/ __ \/ / __/
 / /_/ / /_/ / / /_  
/_.___/\____/_/\__/  v0.1.0
"#
        );

        println!("=> Server running on http://{}", addr);

        let addr: SocketAddr = addr.parse().unwrap();

        let listener = TcpListener::bind(addr).await?;
        let router = Arc::new(self.router.clone());

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            let router = router.clone();
            let error_handler = self.error_handler.clone();

            let service = service_fn(move |req: Request<Incoming>| {
                let router = router.clone();

                let error_handler = error_handler.clone();

                async move {
                    let mut req_body = RequestBody::new(req);
                    let mut res_body = ResponseWriter::new();

                    let method = match *req_body.method() {
                        hyper::Method::GET => Method::GET,
                        hyper::Method::POST => Method::POST,
                        hyper::Method::PUT => Method::PUT,
                        hyper::Method::PATCH => Method::PATCH,
                        hyper::Method::DELETE => Method::DELETE,
                        hyper::Method::OPTIONS => Method::OPTIONS,
                        hyper::Method::HEAD => Method::HEAD,
                        hyper::Method::TRACE => Method::TRACE,
                        _ => {
                            res_body.status(405);
                            res_body.send("Method Not Allowed");
                            return Ok::<_, Infallible>(res_body.into_response());
                        }
                    };

                    let path = req_body.path().to_string();

                    let mws = router.collect_middleware(&path, method);

                    for mw in mws {
                        mw.run(&mut req_body, &mut res_body).await;

                        if res_body.has_error() {
                            break;
                        }
                    }

                    if !res_body.has_error() {
                        if let Some((handler, params)) = router.find(&path, method) {
                            req_body.set_params(params);
                            handler(&mut req_body, &mut res_body);
                        } else {
                            let method_str = match *req_body.method() {
                                hyper::Method::GET => "GET",
                                hyper::Method::POST => "POST",
                                hyper::Method::PUT => "PUT",
                                hyper::Method::PATCH => "PATCH",
                                hyper::Method::DELETE => "DELETE",
                                hyper::Method::OPTIONS => "OPTIONS",
                                hyper::Method::HEAD => "HEAD",
                                hyper::Method::TRACE => "TRACE",
                                _ => "UNKNOWN",
                            };

                            res_body.error(404, &format!("Not Found {} {}", method_str, path));
                        }
                    }

                    if res_body.has_error() {
                        let msg = res_body.body.clone();
                        error_handler.run(msg, &mut res_body).await;
                    }

                    Ok::<_, Infallible>(res_body.into_response())
                }
            });

            match mode {
                Mode::Http1 => {
                    tokio::task::spawn(async move {
                        if let Err(err) = http1::Builder::new().serve_connection(io, service).await
                        {
                            eprintln!("Error serving connection: {}", err);
                        }
                    });
                }

                Mode::Http2 => {
                    tokio::task::spawn(async move {
                        if let Err(err) = http2::Builder::new(TokioExecutor::new())
                            .serve_connection(io, service)
                            .await
                        {
                            eprintln!("Error serving connection: {}", err);
                        }
                    });
                }
            }
        }
    }

    fn add_route(&mut self, method: Method, path: &str, handler: Handler) {
        self.router.insert(path, method, handler);
    }

    pub fn get(&mut self, path: &str, handler: Handler) {
        self.add_route(Method::GET, path, handler);
    }

    pub fn post(&mut self, path: &str, handler: Handler) {
        self.add_route(Method::POST, path, handler);
    }

    pub fn put(&mut self, path: &str, handler: Handler) {
        self.add_route(Method::PUT, path, handler);
    }

    pub fn patch(&mut self, path: &str, handler: Handler) {
        self.add_route(Method::PATCH, path, handler);
    }

    pub fn delete(&mut self, path: &str, handler: Handler) {
        self.add_route(Method::DELETE, path, handler);
    }

    pub fn group(&mut self, path: &str) -> Group {
        Group {
            prefix: path.to_string(),
            app: self,
        }
    }

    pub fn middleware<M>(&mut self, path: &str, method: Option<Method>, middleware_fn: M)
    where
        M: Middleware + 'static,
    {
        let mw = Arc::new(middleware_fn);
        let full_path = path.to_string();

        match method {
            Some(m) => self.router.insert_middleware(&full_path, m, mw),
            None => {
                for m in [
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                    Method::HEAD,
                    Method::TRACE,
                ] {
                    self.router.insert_middleware(&full_path, m, mw.clone());
                }
            }
        }
    }

    pub fn set_error_handler(&mut self, handler: Arc<dyn ErrorHandler>) {
        self.error_handler = handler;
    }
}
