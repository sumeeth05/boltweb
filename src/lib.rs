use futures_util::FutureExt;

use std::{
    convert::Infallible, net::SocketAddr, panic::AssertUnwindSafe, pin::Pin, sync::Arc,
    time::Duration,
};

use hyper::{
    Request,
    body::Incoming,
    server::conn::{http1, http2},
    service::service_fn,
};
use hyper_util::rt::{TokioExecutor, TokioIo};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    sync::Semaphore,
};
use tokio_rustls::TlsAcceptor;

use crate::{
    client::Client,
    error::DefaultErrorHandler,
    group::Group,
    headers::LimitReader,
    http::StatusCode,
    request::RequestBody,
    response::ResponseWriter,
    router::Router,
    tls::tls_config,
    types::{BoltError, ErrorHandler, Handler, Method, Middleware, Mode},
};

pub mod client;
mod error;
mod group;
mod headers;
pub mod http;
pub mod macros;
pub mod request;
pub mod response;
mod router;
mod tls;
pub mod types;
pub use async_trait;
pub use bolt_web_macro::main;
pub use paste;
pub use tokio;

trait Io: AsyncRead + AsyncWrite + Unpin {}
impl<T: AsyncRead + AsyncWrite + Unpin> Io for T {}

#[allow(dead_code)]
pub struct App {
    router: Router,
    error_handler: Arc<dyn ErrorHandler>,
    client: Client,
    timeout: u64,
    connection_limit: u64,
    read_timeout: u64,
    header_limit: usize,
}

#[allow(unused_variables)]
#[allow(dead_code)]
impl App {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            error_handler: Arc::new(DefaultErrorHandler),
            client: Client::new(),
            timeout: 30,
            connection_limit: 100,
            read_timeout: 10,
            header_limit: 32 * 1024,
        }
    }

    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout = seconds;
    }

    pub fn set_connection_limit(&mut self, limit: u64) {
        self.connection_limit = limit;
    }

    pub fn set_read_timeout(&mut self, seconds: u64) {
        self.read_timeout = seconds;
    }

    pub fn set_header_limit(&mut self, bytes: usize) {
        self.header_limit = bytes;
    }

    fn add_route<H>(&mut self, method: Method, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        self.router.insert(path, method, handler);
    }

    pub fn get<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        self.add_route(Method::GET, path, handler);
    }

    pub fn post<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        self.add_route(Method::POST, path, handler);
    }

    pub fn put<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        self.add_route(Method::PUT, path, handler);
    }

    pub fn patch<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        self.add_route(Method::PATCH, path, handler);
    }

    pub fn delete<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static,
    {
        self.add_route(Method::DELETE, path, handler);
    }

    pub fn group<'a>(&'a mut self, path: &str) -> Group<'a> {
        Group {
            prefix: path.to_string(),
            app: self,
        }
    }

    pub fn middleware<M>(&mut self, path: &str, method: Option<Method>, middleware_fn: M)
    where
        M: Middleware + 'static,
    {
        let mw: Arc<M> = Arc::new(middleware_fn);
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

    pub fn set_error_handler<E>(&mut self, handler: E)
    where
        E: ErrorHandler + 'static,
    {
        self.error_handler = Arc::new(handler);
    }

    pub async fn run(&self, addr: &str, mode: Mode) -> Result<(), BoltError> {
        println!("⚡ A high performance & minimalist web framework in rust.");
        println!(
            r#"
    __          ____
   / /_  ____  / / /_
  / __ \/ __ \/ / __/
 / /_/ / /_/ / / /_  
/_.___/\____/_/\__/  v0.2.0
"#
        );

        println!(">> Server running on http://{}", addr);

        let addr: SocketAddr = addr.parse().unwrap();

        let listener = TcpListener::bind(addr).await?;
        let router = Arc::new(self.router.clone());
        let error_handler = self.error_handler.clone();
        let active = Arc::new(Semaphore::new(self.connection_limit as usize));

        self.server_loop(
            router,
            error_handler,
            listener,
            mode,
            None,
            self.timeout,
            self.read_timeout,
            Box::pin(tokio::signal::ctrl_c().map(|_| ())),
            active,
        )
        .await
    }

    pub async fn run_tls(
        &self,
        addr: &str,
        mode: Mode,
        tls: Option<(&str, &str)>,
    ) -> Result<(), BoltError> {
        println!("⚡ A high performance & minimalist web framework in rust.");
        println!(
            "{}",
            r#"
    __          ____
   / /_  ____  / / /_
  / __ \/ __ \/ / __/
 / /_/ / /_/ / / /_  
/_.___/\____/_/\__/  v0.2.0
"#
        );

        let addr: SocketAddr = addr.parse().unwrap();
        let listener = TcpListener::bind(addr).await?;

        let tls_acceptor: Option<Arc<TlsAcceptor>> = if let Some((cert, key)) = tls {
            let cfg = tls_config(cert, key)?;
            Some(Arc::new(TlsAcceptor::from(cfg)))
        } else {
            None
        };

        println!(
            ">> Server running on {}://{}",
            if tls_acceptor.is_some() {
                "https"
            } else {
                "http"
            },
            addr
        );

        let router: Arc<Router> = Arc::new(self.router.clone());
        let error_handler = self.error_handler.clone();
        let active = Arc::new(Semaphore::new(self.connection_limit as usize));

        self.server_loop(
            router,
            error_handler,
            listener,
            mode,
            tls_acceptor,
            self.timeout,
            self.read_timeout,
            Box::pin(tokio::signal::ctrl_c().map(|_| ())),
            active,
        )
        .await
    }

    async fn server_loop(
        &self,
        router: Arc<Router>,
        error_handler: Arc<dyn ErrorHandler>,
        listener: TcpListener,
        mode: Mode,
        tls_acceptor: Option<Arc<TlsAcceptor>>,
        timeout: u64,
        read_timeout: u64,
        mut shutdown: Pin<Box<dyn Future<Output = ()> + Send>>,
        active: Arc<Semaphore>,
    ) -> Result<(), BoltError> {
        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    println!(">> Shutdown signal received. Stopping server...");
                    break;
                }

                accept_res = listener.accept() => {
                    let (stream, remote_addr) = match accept_res {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Accept error: {}", e);
                            continue;
                        }
                    };

                    let permit = match active.clone().try_acquire_owned() {
                        Ok(p) => p,
                        Err(_) => {
                            eprintln!("Connection limit reached — dropping client");
                            continue;
                        }
                    };

                    let io: Box<dyn Io + Send> = if let Some(ref acceptor) = tls_acceptor {
                        match acceptor.accept(stream).await {
                            Ok(c) => Box::new(c),
                            Err(e) => {
                                eprintln!("TLS error: {}", e);
                                continue;
                            }
                        }
                    } else {
                        Box::new(stream)
                    };

                    let limited = LimitReader::new(io, self.header_limit);
                    let io = TokioIo::new(limited);

                    let router = router.clone();
                    let error_handler = error_handler.clone();

                    let service = service_fn(move |req: Request<Incoming>| {
                        let router = router.clone();
                        let error_handler = error_handler.clone();
                        let remote_addr = remote_addr.clone();
                        let timeout = timeout;

                        async move {
                            let handler_future = tokio::time::timeout(
                                Duration::from_secs(timeout),
                                async {
                                    let inner = AssertUnwindSafe(async move {
                                        let mut req_body = RequestBody::new(req, remote_addr);
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
                                                res_body.status(StatusCode::MethodNotAllowed)
                                                        .send("Method Not Allowed");
                                                return res_body;
                                            }
                                        };

                                        let path = req_body.path().to_string();
                                        for mw in router.collect_middleware(&path, method) {
                                            mw.run(&mut req_body, &mut res_body).await;
                                            if res_body.has_error() { break; }
                                        }

                                        if !res_body.has_error() {
                                            if let Some((handler, params)) = router.find(&path, method) {
                                                req_body.set_params(params);
                                                handler.run(&mut req_body, &mut res_body).await;
                                            } else {
                                                res_body.error(
                                                    StatusCode::NotFound,
                                                    &format!("Not Found {} {}", req_body.method(), path),
                                                );
                                            }
                                        }

                                        if res_body.has_error() {
                                            let msg = res_body.body.clone();
                                            error_handler.run(msg, &mut res_body).await;
                                        }

                                        req_body.cleanup().await;
                                        res_body
                                    })
                                    .catch_unwind()
                                    .await;

                                    match inner {
                                        Ok(r) => r,
                                        Err(_) => {
                                            let mut res = ResponseWriter::new();
                                            res.error(StatusCode::InternalServerError, "Internal Server Error");
                                            res
                                        }
                                    }
                                }
                            ).await;

                            let res_body = match handler_future {
                                Ok(res) => res,
                                Err(_) => {
                                    let mut res = ResponseWriter::new();
                                    res.error(StatusCode::RequestTimeout, "Request Timeout");
                                    res
                                }
                            };

                            Ok::<_, Infallible>(res_body.into_response())
                        }
                    });

                    let permit = permit;

                    match mode {
                        Mode::Http1 => {
                            tokio::spawn(async move {
                                let _permit = permit;

                                let result = tokio::time::timeout(
                                    Duration::from_secs(read_timeout),
                                    async {
                                        http1::Builder::new()
                                            .serve_connection(io, service)
                                            .await
                                    }
                                ).await;

                                match result {
                                    Ok(Ok(_)) => {}
                                    Ok(Err(e)) => eprintln!("Connection error: {}", e),
                                    Err(_) => eprintln!("Slowloris: read timeout — closing connection"),
                                }
                            });
                        }

                        Mode::Http2 => {
                            tokio::spawn(async move {
                                let _permit = permit;

                                let result = tokio::time::timeout(
                                    Duration::from_secs(read_timeout),
                                    async {
                                        http2::Builder::new(TokioExecutor::new())
                                            .serve_connection(io, service)
                                            .await
                                    }
                                ).await;

                                match result {
                                    Ok(Ok(_)) => {}
                                    Ok(Err(e)) => eprintln!("Connection error: {}", e),
                                    Err(_) => eprintln!("Slowloris: read timeout — closing connection"),
                                }
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
