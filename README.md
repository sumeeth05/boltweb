# ⚡ Bolt-Web

**A high-performance, minimalist web framework for Rust — inspired by Express.js and Gin.**

Bolt is a lightweight, modular, and fully asynchronous web framework built on top of
[`hyper`](https://github.com/hyperium/hyper) and [`tokio`](https://tokio.rs/).
Its goal is **performance, simplicity, and control** — perfect for REST APIs, microservices, and backend systems.

---

## 🚀 Features

- 🌐 **HTTP/1.x & HTTP/2 Support** — Built-in protocol selection.
- 🚦 **Fast Router** — Path params, wildcards, and deterministic matching.
- 🧩 **Middleware System** — CORS, Helmet, Logging, Rate-Limiting, and more.
- 🔄 **Async-First** — Everything is async, from routing to middleware.
- 👥 **Route Groups** — Clean organization for large APIs.
- 🔒 **Security Built-in** — Panic protection, timeouts, connection limits, header/body limits.
- 🌐 **Minimal HTTP Client** — Useful for internal service calls.

---

## 📦 Dependencies

```toml
[dependencies]
bolt-web = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"
```

---

## 🦀 Basic Example

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

    app.run("127.0.0.1:8080", Mode::Http1, None).await?;
    Ok(())
}

async fn hello(_: &mut RequestBody, res: &mut ResponseWriter) {
    res.json(&json!({ "msg": "hello" }));
}
```

---

## 🧭 Routing

Bolt offers a clean and expressive routing system.
Route macros like `Get!`, `Post!`, `Put!`, etc., automatically generate handler types.

### Basic route

```rust
Get!(app, "/hello", hello);

async fn hello(_req: &mut RequestBody, res: &mut ResponseWriter) {
    res.send("Hello, world!");
}
```

### Path Parameters

```rust
Get!(app, "/users/:id", get_user);

async fn get_user(req: &mut RequestBody, res: &mut ResponseWriter) {
    let id = req.param("id");
    res.send(&format!("User ID: {}", id));
}
```

### Wildcard

```rust
Get!(app, "/files/*path", get_file);
```

### Query Parameters

```rust
let page = req.query_param("page").unwrap_or("1".into());
```

---

## 🗂 Route Groups

```rust
let mut api = app.group("/api");

api.get("/status", status);
api.post("/login", login);

let mut v1 = api.group("/v1");
v1.get("/users", list_users);
```

Groups make large APIs clean and maintainable.

---

## 🔧 Middleware

Middleware can run **before handlers** and can short-circuit responses.

```rust
async fn log(req: &mut RequestBody, _res: &mut ResponseWriter) {
    println!("{} {}", req.method(), req.path());
}

Middleware!(app, "/", log);
```

---

## 🍪 Cookies

Bolt uses the `cookie` crate to generate RFC-compliant cookies.

```rust
res.cookie(
    "session", "abc123",
    Some(3600),         // 1 hour
    Some("/"),
    None,
    true,               // Secure
    true,               // HttpOnly
    Some("lax")
);
```

---

## 🌐 HTTP Client

Bolt includes a minimal async HTTP client for external APIs.

```rust
use bolt_web::Client;

let client = Client::new();

let joke: Joke = client.get("https://icanhazdadjoke.com", &None).await?;
```

---

## 🛡 Security

Bolt includes multiple production-grade protections:

- Panic isolation
- Request timeout
- Read timeout (Slowloris protection)
- Header limits
- Body size limits
- Connection limits
- Graceful shutdown
- TLS support

---

## 🧠 License

MIT © 2025 — Built with ❤️ in Rust.
