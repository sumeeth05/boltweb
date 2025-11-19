# Bolt-web

⚡ A high-performance, minimalist web framework for Rust, inspired by Express.js and Gin.

**Bolt** is a lightweight, modular, and fully asynchronous web framework built on top of [`hyper`](https://github.com/hyperium/hyper) and [`tokio`](https://tokio.rs/).  
It focuses on **performance, simplicity**, and **full control** — ideal for REST APIs, WebSocket services, and microservice backends.

---

## 🚀 Features

- 🌐 **Supports HTTP/1.x and HTTP/2** - built in support for both http/1.x and http/2.
- 🔥 **Built in Router** - Fast and flexible routing system with path parameters, dynamic segments, and middleware chaining per route.
- ⚙️ **Grouping Routes** — Simple builder-style API for organizing endpoints.
- 🧩 **Middleware System** — Add CORS, Helmet, Logging, Rate Limiting, Error handling easily.
- 🔄 **Fully Supports Async** — Built on top of [`tokio`](https://tokio.rs/).
- ⚙️ **Request & Response Abstraction** — Simple builder-style API for request/response.
- 🌍 **Minimal HTTP Client** — Builtin client for inter-service communication (OAuth, APIs, etc).

---

## Dependencies

```rust

    [dependencies]
    bolt-web = "0.2"
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
    Get,
};

#[bolt_web::main]
async fn main() -> BoltResult<()> {
    let mut app = Bolt::new();

    Get!(app, "/hello", hello);

    app.run("127.0.0.1:8080", Mode::Http1, None).await.unwrap();
    Ok(())
}

async fn hello(_: &mut RequestBody, res: &mut ResponseWriter) {
    res.json(&json!({
        "msg" : "hello"
    }));
}


```

## 🧭 Routing

```rust
Get!(app, "/users", list_users);
Post!(app, "/users", create_user);

let mut api = app.group("/api");
api.get("/health", health_check);
api.post("/login", login_user);
```

Path params:

```rust
async fn get_user(req: &mut RequestBody, res: &mut ResponseWriter) {
    let id = req.param("id");
    res.send(&format!("User: {}", id));
}
```

## 🔧 Middleware

```rust
async fn log(req: &mut RequestBody, _res: &mut ResponseWriter) {
    println!("{} {}", req.method(), req.path());
}

Middleware!(app, "/", log);
```

##

```rust
res.cookie(
    "session", "abc123",
    Some(3600),         // 1 hour
    Some("/"),
    None,
    true,               // secure
    true,               // httpOnly
    Some("lax")
);
```

## ⚡ HTTP Client Example

Use the built-in Client to make external API calls.

```rust
use bolt_web::Client;

let client = Client::new();

let joke: Joke = client.get("https://icanhazdadjoke.com/").await.unwrap();
```

**🧠 License**

MIT © 2025 — Built with ❤️ in Rust.
