# brakes

**brakes** is a distributed rate limiting library. It offers a number of rate limiting algorithms, supports multiple caching backends (local memory, Redis, Memcached), and includes a set of middlewares for popular Rust web frameworks like [Actix Web](https://actix.rs/) and [Axum](https://docs.rs/axum/latest/axum/).

## Features
- Support for multiple rate limiting algorithms:
  - Fixed window
  - Sliding window counter
  - Token bucket
  - Leaky bucket
- Configurable caching backends:
  - Local memory
  - Memcache
  - Redis
- Middleware for popular frameworks (see examples):
  - [Actix Web](https://actix.rs/)
  - [Axum](https://docs.rs/axum/latest/axum/)
- Retry strategies

## Usage

### You can use `RateLimiter` directly

```rust
use std::time::Duration;

use brakes::{
    backend::local::Memory,
    types::{leaky_bucket::LeakyBucket, RateLimiterError},
    RateLimiter,
};

fn main() {
    let limiter = RateLimiter::builder()
        .with_backend(Memory::new())
        .with_limiter(LeakyBucket::new(100, Duration::from_secs(10)))
        .build();

    let result = limiter.is_ratelimited("key");
    match &result {
        Ok(()) => println!("allowed"),
        Err(RateLimiterError::RateExceeded) => println!("rate exceeded"),
        Err(e) => println!("error {:?}", e),
    }
    
    assert!(result.is_ok());
}
```

### Built-in middlewares

#### Actixweb:

```rust
use std::time::Duration;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use brakes::{
    backend::memcache::MemCache, middleware::actixweb::ActixwebRateLimiter,
    types::token_bucket::TokenBucket, RateLimiter,
};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let cache = memcache::connect("memcache://127.0.0.1:11211").unwrap();

    let hello_limiter = RateLimiter::builder()
        .with_backend(MemCache::new(cache.clone()))
        .with_limiter(TokenBucket::new(2, Duration::from_secs(2)))
        .build();

    let hello_middleware = ActixwebRateLimiter::new(hello_limiter);

    let echo_limiter = RateLimiter::builder()
        .with_backend(MemCache::new(cache))
        .with_limiter(TokenBucket::new(5, Duration::from_secs(1)))
        .build();

    let echo_middleware = ActixwebRateLimiter::new(echo_limiter)
        .with_callback(|_| HttpResponse::TooManyRequests().body("too many requests"))
        .with_key_extractor(|req| {
            req.headers()
                .get("x-forwarded-for")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        });

    HttpServer::new(move || {
        let hello_middleware = hello_middleware.clone();
        let echo_middleware = echo_middleware.clone();

        App::new()
            .service(web::scope("hello").wrap(hello_middleware).service(hello))
            .service(web::scope("echo").wrap(echo_middleware).service(echo))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

```

#### Axum

Axum doesn't have a middleware system of its own, instead it relies on `tower` middleware

```rust
use std::{net::SocketAddr, time::Duration};

use axum::{body::Body, extract::ConnectInfo, routing::get, Router};
use brakes::{
    backend::redis::RedisBackend, middleware::tower::TowerRateLimiterLayer,
    types::fixed_window::FixedWindow, RateLimiter,
};

async fn hello() -> &'static str {
    "Hello, World!"
}

async fn hi() -> &'static str {
    "hi"
}

#[tokio::main]
async fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .connection_timeout(Duration::from_secs(1))
        .build(client)
        .unwrap();

    let hello_limiter = RateLimiter::builder()
        .with_backend(RedisBackend::new(pool.clone()))
        .with_limiter(FixedWindow::new(5, Duration::from_secs(10)))
        .build();

    let hello_layer =
        // ::default()  uses the default callback
        TowerRateLimiterLayer::default(hello_limiter, |r: &axum::http::Request<Body>| {
            // key extractor
            r.headers()
                .get("x-forwarded-for")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        });

    let hi_limiter = RateLimiter::builder()
        .with_backend(RedisBackend::new(pool))
        .with_limiter(FixedWindow::new(5, Duration::from_secs(10)))
        .build();

    let hi_layer = TowerRateLimiterLayer::new(
        hi_limiter,
        // callback for RateExceeded
        |_| {
            axum::response::Response::builder()
                .status(429)
                .body(Body::from("too many requests"))
                .unwrap()
        },
        // key extractor
        |r: &axum::http::Request<Body>| {
            r.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .unwrap()
                .ip()
                .to_string()
        },
    );

    let app = Router::new()
        .route("/hello", get(hello).layer(hello_layer))
        .route("/hi", get(hi).layer(hi_layer));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

```
