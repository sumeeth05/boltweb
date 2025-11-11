# Bolt-web

⚡ A high-performance, minimalist web framework for Rust, inspired by Express.js and Gin.

**Bolt** is a lightweight, modular, and fully asynchronous web framework built on top of [`hyper`](https://github.com/hyperium/hyper) and [`tokio`](https://tokio.rs/).  
It focuses on **performance, simplicity**, and **full control** — ideal for REST APIs, WebSocket services, and microservice backends.

---

## 🚀 Features

- 🌐 **Supports HTTP/1.x and HTTP/2** - built in support for both http/1.x and http/2.
- 🔥 **Built in Router** - Fast and flexible routing system with path parameters, dynamic segments, and middleware chaining per route.
- ⚙️ **Grouping Routes** — Simple builder-style API for responses.
- 🧩 **Middleware System** — Add CORS, Helmet, Logging, Rate Limiting, Error handling easily.
- 🔄 **Fully Supports Async** — Built on top of [`tokio`](https://tokio.rs/).
- ⚙️ **Request & Response Abstraction** — Simple builder-style API for responses.
- 🌍 **Minimal HTTP Client** — Builtin client for inter-service communication (OAuth, APIs, etc).

---

## Dependencies

```rust

    [dependencies]
    bolt-web = "0.2.7"
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1"

```

## 🦀 Example Usage

```rust
use serde_json::json;

use bolt_web::{
    Bolt,
    request::RequestBody,
    response::ResponseWriter,
    types::{BoltResult, Mode},
};

#[bolt_web::main]
async fn main() -> BoltResult<()> {
    let mut app = Bolt::new();

    app.get("/", HelloHandler);

    app.run("127.0.0.1:8080", Mode::Http1, None).await.unwrap();
    Ok(())
}

async fn hello(_: &mut RequestBody, res: &mut ResponseWriter) {
    res.json(&json!({
        "msg" : "hello"
    }));
}

bolt_handler!(hello);

```

## ⚡ HTTP Client Example

Use the built-in Client to make external API calls.

```rust
use bolt_web::Client;

let client = Client::new();

let joke: Joke = client.get("https://icanhazdadjoke.com/").await.unwrap();

```

## 🔧 Middleware

Comes with helpful middleware by default.

`Logger` Prints method and route for every request.
`Helmet` Sets secure HTTP headers.  
`Cors` Enables cross-origin requests.  
`RateLimiter` Simple in-memory request limiter.  
`ErrorHandler` Handles and serializes errors.

**🧠 License**

MIT © 2025 — Built with ❤️ in Rust.
