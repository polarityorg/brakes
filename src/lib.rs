#![cfg_attr(docsrs, feature(doc_cfg))]
//! # brakes
//!
//! **brakes** is a distributed rate limiting library. It offers a number of rate limiting algorithms, supports multiple caching backends (local memory, Redis, Memcached), and includes a set of middlewares for popular Rust web frameworks like [Actix Web](https://actix.rs/) and [Axum](https://docs.rs/axum/latest/axum/).
//!
//! ## Features
//! - Support for multiple rate limiting algorithms:
//!   - Fixed window
//!   - Sliding window counter
//!   - Token bucket
//!   - Leaky bucket
//! - Configurable caching backends:
//!   - Local memory
//!   - Memcache
//!   - Redis
//! - Middleware for popular frameworks (see examples):
//!   - [Actix Web](https://actix.rs/)
//!   - [Axum](https://docs.rs/axum/latest/axum/)
//! - Retry strategies
//!
//! ## Usage
//!
//! ### You can use `RateLimiter` directly
//!
//! ```rust
//! use std::time::Duration;
//!
//! use brakes::{
//!     backend::local::Memory,
//!     types::{leaky_bucket::LeakyBucket, RateLimiterError},
//!     RateLimiter,
//! };
//!
//! fn main() {
//!     let limiter = RateLimiter::builder()
//!         .with_backend(Memory::new())
//!         .with_limiter(LeakyBucket::new(100, Duration::from_secs(10)))
//!         .build();
//!
//!     let result = limiter.is_ratelimited("key");
//!     match &result {
//!         Ok(()) => println!("allowed"),
//!         Err(RateLimiterError::RateExceeded) => println!("rate exceeded"),
//!         Err(e) => println!("error {:?}", e),
//!     }
//!     
//!     assert!(result.is_ok());
//! }
//! ```
//!
//! ### Built-in middlewares
//!
//! #### Actixweb:
//!
//! **Available on crate feature `actixweb` only**
//!
//! ```rust,ignore
//! use std::time::Duration;
//!
//! use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
//! use brakes::{
//!     backend::memcache::MemCache, middleware::actixweb::ActixwebRateLimiter,
//!     types::token_bucket::TokenBucket, RateLimiter,
//! };
//!
//! #[get("/")]
//! async fn hello() -> impl Responder {
//!     HttpResponse::Ok().body("Hello world!")
//! }
//!
//! #[post("/")]
//! async fn echo(req_body: String) -> impl Responder {
//!     HttpResponse::Ok().body(req_body)
//! }
//!
//! #[actix_web::main]
//! async fn main() -> Result<(), std::io::Error> {
//!     let cache = memcache::connect("memcache://127.0.0.1:11211").unwrap();
//!
//!     let hello_limiter = RateLimiter::builder()
//!         .with_backend(MemCache::new(cache.clone()))
//!         .with_limiter(TokenBucket::new(2, Duration::from_secs(2)))
//!         .build();
//!
//!     let hello_middleware = ActixwebRateLimiter::new(hello_limiter);
//!
//!     let echo_limiter = RateLimiter::builder()
//!         .with_backend(MemCache::new(cache))
//!         .with_limiter(TokenBucket::new(5, Duration::from_secs(1)))
//!         .build();
//!
//!     let echo_middleware = ActixwebRateLimiter::new(echo_limiter)
//!         .with_callback(|_| HttpResponse::TooManyRequests().body("too many requests"))
//!         .with_key_extractor(|req| {
//!             req.headers()
//!                 .get("x-forwarded-for")
//!                 .unwrap()
//!                 .to_str()
//!                 .unwrap()
//!                 .to_string()
//!         });
//!
//!     HttpServer::new(move || {
//!         let hello_middleware = hello_middleware.clone();
//!         let echo_middleware = echo_middleware.clone();
//!
//!         App::new()
//!             .service(web::scope("hello").wrap(hello_middleware).service(hello))
//!             .service(web::scope("echo").wrap(echo_middleware).service(echo))
//!     })
//!     .bind(("127.0.0.1", 8080))?
//!     .run()
//!     .await
//! }
//!
//! ```
//!
//! #### Axum
//!
//! **Available on crate feature `tower` only**
//!
//! Axum doesn't have a middleware system of its own, instead it relies on `tower` middleware
//!
//! ```rust,ignore
//! use std::{net::SocketAddr, time::Duration};
//!
//! use axum::{body::Body, extract::ConnectInfo, routing::get, Router};
//! use brakes::{
//!     backend::redis::RedisBackend, middleware::tower::TowerRateLimiterLayer,
//!     types::fixed_window::FixedWindow, RateLimiter,
//! };
//!
//! async fn hello() -> &'static str {
//!     "Hello, World!"
//! }
//!
//! async fn hi() -> &'static str {
//!     "hi"
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = redis::Client::open("redis://127.0.0.1/").unwrap();
//!     let pool = r2d2::Pool::builder()
//!         .connection_timeout(Duration::from_secs(1))
//!         .build(client)
//!         .unwrap();
//!
//!     let hello_limiter = RateLimiter::builder()
//!         .with_backend(RedisBackend::new(pool.clone()))
//!         .with_limiter(FixedWindow::new(5, Duration::from_secs(10)))
//!         .build();
//!
//!     let hello_layer =
//!         // ::default()  uses the default callback
//!         TowerRateLimiterLayer::default(hello_limiter, |r: &axum::http::Request<Body>| {
//!             // key extractor
//!             r.headers()
//!                 .get("x-forwarded-for")
//!                 .unwrap()
//!                 .to_str()
//!                 .unwrap()
//!                 .to_string()
//!         });
//!
//!     let hi_limiter = RateLimiter::builder()
//!         .with_backend(RedisBackend::new(pool))
//!         .with_limiter(FixedWindow::new(5, Duration::from_secs(10)))
//!         .build();
//!
//!     let hi_layer = TowerRateLimiterLayer::new(
//!         hi_limiter,
//!         // callback for RateExceeded
//!         |_| {
//!             axum::response::Response::builder()
//!                 .status(429)
//!                 .body(Body::from("too many requests"))
//!                 .unwrap()
//!         },
//!         // key extractor
//!         |r: &axum::http::Request<Body>| {
//!             r.extensions()
//!                 .get::<ConnectInfo<SocketAddr>>()
//!                 .unwrap()
//!                 .ip()
//!                 .to_string()
//!         },
//!     );
//!
//!     let app = Router::new()
//!         .route("/hello", get(hello).layer(hello_layer))
//!         .route("/hi", get(hi).layer(hi_layer));
//!
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
//!         .await
//!         .unwrap();
//!     axum::serve(
//!         listener,
//!         app.into_make_service_with_connect_info::<SocketAddr>(),
//!     )
//!     .await
//!     .unwrap();
//! }
//!
//! ```
//!
//! ## Cache Backends
//! Cache backends are used to store `LimiterInstance`s. A `LimiterInstance` contains information about a single rate limiter instance's (a user's or ip's) usage.
//!
//! ### Memory
//! Uses an in memory `HashMap` to store keys and values (`LimiterInstance`s).
//!
//! It can be used safely across threads since it utilizes a `Mutex`, but it can't be used across processes or in a distributed fashion.
//!
//! ```rust,ignore
//! let memory_cache = Memory::new();
//! let limiter = RateLimiter::builder()
//!     .with_backend(memory_cache)
//!     .with_limiter(...)
//!     .build();
//! ```
//!
//! ### Memcache
//!
//! **Available on crate feature `memcache` only**
//!
//! Uses a `memcache` client to store `LimiterInstance` data.
//!
//! Writes to `memcache` are done using the `CAS` (check-and-set) command to ensure conncurent writes won't conflict.
//!
//! If there's a conflict (data related to a single `LimiterInstance` changed while it was being updated by another process), the write is either retried (if `RetryAndAllow` or `RetryAndDeny` is used) or a `RateLimiterError::BackendConflict` is returned. In either case, whether the request is ratelimited or not is based on the `RetryStrategy` used.
//!
//! ```rust,ignore
//! let cache = memcache::connect("memcache://127.0.0.1:11211").unwrap();
//! let memcache_backend = MemCache::new(cache);
//! let limiter = RateLimiter::builder()
//!     .with_backend(memcache_backend)
//!     .with_limiter(FixedWindow::new(10000, Duration::from_millis(1000)))
//!     .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(2))
//!     .build();
//! ```
//!
//! ### Redis
//!
//! **Available on crate feature `redis` only**
//!
//! Uses a `redis` connection pool to connect to redis. `RedisBackend::new` expects a `r2d2::Pool`.
//!
//! Writes use `transactions` (`WATCH`, `MULTI`, and `EXEC`) to ensure conncurent writes won't conflict.
//!
//! If there's a conflict (data related to a single `LimiterInstance` changed while it was being updated by another process), the write is either retried (if `RetryAndAllow` or `RetryAndDeny` is used) or a `RateLimiterError::BackendConflict` is returned. In either case, whether the request is ratelimited or not is based on the `RetryStrategy` used.
//!
//! ```rust,ignore
//! let client = redis::Client::open("redis://127.0.0.1/").unwrap();
//! let pool = r2d2::Pool::builder().build(client).unwrap();
//!     
//! let limiter = RateLimiter::builder()
//!     .with_backend(RedisBackend::new(pool))
//!     .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
//!     .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
//!     .build();
//! ```
//!
//! ## Rate Limiter Types
//!
//! `LimiterType` dictates the rate limiting algorithm to be used.
//!
//! The `LimiterType` (ex: `FixedWindow` limiter type) stores configuration about the algorithm (ex: for `FixedWindow`, it's the `threshold` and the `window_size`), while its associated `LimiterInstance` stores information about a single key's (user, for example) usage of the limiter (ex: for `FixedWindow`, it's `window_start` timestamp and `count`).
//!
//! `LimiterInstance`s are stored in the configured `Backend`
//!
//! ### FixedWindow
//! Defined by a `threshold` and a `window_length`.
//!
//! The `FixedWindowInstance` keeps track of `window_start` and `count` for each key (user, for example).
//!
//! ```rust,ignore
//! // allow upto 10 requests in any 1000ms fixed window.
//! let limiter = RateLimiter::builder()
//!     .with_backend(...)
//!     .with_limiter(FixedWindow::new(10, Duration::from_millis(1000)))
//!     .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
//!     .build();
//! ```
//!
//! ### SlidingWindowCounter
//! Defined by a `threshold` and a `window_length`.
//!
//! The `SlidingWindowInstance` keeps track of the current and previous `window_start` and `count`.
//!
//! ```rust,ignore
//! // allow upto 5 requests in any 1000ms sliding window.
//! let limiter = RateLimiter::builder()
//!     .with_backend(...)
//!     .with_limiter(SlidingWindowCounter::new(5, Duration::from_millis(1000)))
//!     .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
//!     .build();
//! ```
//!
//! ### TokenBucket
//! Defined by a `capacity` and a `fill_frequency`.
//!
//! A bucket with a `capacity` of 10, and a `fill_frequency` of 1 second will allow up to 10 requests to be allowed. Each request consumes a token from the bucket. The bucket is refilled by 1 token every second. If the bucket is empty, no requests are allowed.
//!
//! The `TokenBucketInstance` keeps track of how many `token`s are available and the `last_access` timestamp for the user.
//!
//! ```rust,ignore
//! // 10 tokens at most, with a fill rate of 1 token every 2 seconds
//!let hello_limiter = RateLimiter::builder()
//!    .with_backend(...)
//!    .with_limiter(TokenBucket::new(10, Duration::from_secs(2)))
//!    .build();
//! ```
//!
//! ### LeakyBucket
//! Defined by a `capacity` and a `leak_frequency`.
//!
//! A bucket with a `capacity` of 10, and a `leak_frequency` of 1 second will allow up to 10 requests to be allowed. Each request is added to the bucket until it's full. If the bucket is full, further requests are denied until requests are leaked. A `leak_frequency` of 1 second will leak one request per second.
//!
//! The `LeakyBucketInstance` keeps track of how many allowed requests there are in the bucket and the `last_leaked` timestamp for the user.
//!
//! ```rust,ignore
//! // upto 100 requests can be allowed, with a leak rate of 1 request every 2 seconds
//!let hello_limiter = RateLimiter::builder()
//!    .with_backend(...)
//!    .with_limiter(LeakyBucket::new(100, Duration::from_secs(2)))
//!    .build();
//! ```
//!
//! ## Retry Strategies
//!
//! Retry strategies can be useful in two cases:
//! - When reads or writes to the `Backend` fail (for example due to a network timeout). Can be set using `RateLimiterBuilder::with_failure_strategy`
//! - When writes to the `Backend` fail due to a conflict (caused by concurrent requests for the same ip for example). Can be set using `RateLimiterBuilder::with_conflict_strategy`
//!
//! A `RetyStrategy` can be one of four:
//! - `RetryAndAllow(n)` tries the operation a total of n+1 times. If all fail, it allows the request.
//! - `RetryAndDeny(n)` tries the operation a total of n+1 times. If all fail, it denies the request.
//! - `Allow` allows the request without retries.
//! - `Deny` denies the request without retries.
//!
//! Where `n` is the number of retries.
//!
//! If `with_failure_strategy` or `with_conflict_strategy` is not set, the default is used:
//! - Failure strategy of `RetryStrategy::RetryAndAllow(2)`
//! - Conflict strategy of `RetryStrategy::RetryAndDeny(2)`
//!
//! Both can be set as follows:
//!
//! ```rust,ignore
//! let limiter = RateLimiter::builder()
//!     .with_backend(...)
//!     .with_limiter(...)
//!     .with_failure_strategy(brakes::RetryStrategy::RetryAndAllow(1))
//!     .with_conflict_strategy(brakes::RetryStrategy::Deny)
//!     .build();
//! ```
//!
pub mod backend;
pub mod middleware;
pub mod types;

use crate::{
    backend::{Backend, BackendError},
    types::LimiterType,
};
use types::{LimiterInstance, RateLimiterError};

#[derive(Debug, Clone)]
pub struct RateLimiter<T, B> {
    limiter: T,
    backend: B,
    on_failure: RetryStrategy,
    on_conflict: RetryStrategy,
    hasher: Option<fn(&str) -> String>,
}

impl<T: LimiterType, B: Backend> RateLimiter<T, B> {
    pub fn builder() -> RateLimiterBuilder<T, B> {
        RateLimiterBuilder {
            backend: None,
            limiter: None,
            on_failure: None,
            on_conflict: None,
            hasher: None,
        }
    }

    pub fn is_ratelimited(&self, key: &str) -> Result<(), RateLimiterError> {
        let key = match self.hasher {
            Some(h) => &(h)(key),
            None => key,
        };

        let (failure_tries, allow_on_failure) = match self.on_failure {
            RetryStrategy::RetryAndAllow(retries) => (retries + 1, true),
            RetryStrategy::RetryAndDeny(retries) => (retries + 1, false),
            RetryStrategy::Allow => (1, true),
            RetryStrategy::Deny => (1, false),
        };
        let (conflict_tries, allow_on_conflict) = match self.on_conflict {
            RetryStrategy::RetryAndAllow(retries) => (retries + 1, true),
            RetryStrategy::RetryAndDeny(retries) => (retries + 1, false),
            RetryStrategy::Allow => (1, true),
            RetryStrategy::Deny => (1, false),
        };

        for _ in 0..conflict_tries {
            let (value, version) = match self.backend.get_with_retries(&key, failure_tries) {
                Ok((value, version)) => (Some(value), version),
                Err(BackendError::KeyMissing) => (None, None),
                Err(e) => {
                    if allow_on_failure {
                        return Ok(());
                    }
                    return Err(RateLimiterError::BackendError(e));
                }
            };
            let updated_limiter = self.limiter.is_ratelimited(value);
            match updated_limiter {
                Ok(v) => match self
                    .backend
                    .set_with_retries(&key, v, version, failure_tries)
                {
                    Ok(()) => return Ok(()),
                    Err(BackendError::ValueChanged) => {
                        continue;
                    }
                    Err(e) => {
                        if allow_on_failure {
                            return Ok(());
                        }
                        return Err(RateLimiterError::BackendError(e));
                    }
                },
                Err(e) => return Err(e),
            }
        }
        if allow_on_conflict {
            return Ok(());
        }
        Err(RateLimiterError::BackendConflict)
    }

    pub fn get_usage(&self, key: &str) -> Result<LimiterInstance, RateLimiterError> {
        let value = match self.backend.get(&key) {
            Ok((v, _)) => v,
            Err(e) => return Err(RateLimiterError::BackendError(e)),
        };
        self.limiter.window_instance(value)
    }
}

pub struct RateLimiterBuilder<C, B> {
    backend: Option<B>,
    limiter: Option<C>,
    on_failure: Option<RetryStrategy>,
    on_conflict: Option<RetryStrategy>,
    hasher: Option<fn(&str) -> String>,
}

impl<C, B> RateLimiterBuilder<C, B>
where
    C: LimiterType,
    B: Backend,
{
    pub fn with_backend(mut self, backend: B) -> Self {
        self.backend = Some(backend);
        self
    }

    pub fn with_limiter(mut self, class: C) -> Self {
        self.limiter = Some(class);
        self
    }

    pub fn with_failure_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.on_failure = Some(strategy);
        self
    }

    pub fn with_conflict_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.on_conflict = Some(strategy);
        self
    }

    pub fn with_hasher(mut self, hasher: fn(&str) -> String) -> Self {
        self.hasher = Some(hasher);
        self
    }

    pub fn build(mut self) -> RateLimiter<C, B> {
        if self.backend.is_none() {
            panic!("no backend specified");
        }
        if self.limiter.is_none() {
            panic!("no limiter specified");
        }

        if self.on_failure.is_none() {
            self.on_failure = Some(RetryStrategy::RetryAndAllow(2))
        }

        if self.on_conflict.is_none() {
            self.on_conflict = Some(RetryStrategy::RetryAndDeny(2))
        }

        RateLimiter {
            backend: self.backend.unwrap(),
            limiter: self.limiter.unwrap(),
            on_failure: self.on_failure.unwrap(),
            on_conflict: self.on_conflict.unwrap(),
            hasher: self.hasher,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    RetryAndAllow(u32),
    RetryAndDeny(u32),
    Allow,
    Deny,
}
