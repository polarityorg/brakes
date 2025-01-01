use crate::{backend::Backend, types::LimiterType, RateLimiter};
use futures::future::{ready, Either, Ready};
use http::{Request, Response, StatusCode};
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct TowerRateLimiter<S, T, B, F, K> {
    inner: S,
    limiter: RateLimiter<T, B>,
    callback: F,
    key_extractor: K,
}

impl<S, T: LimiterType, B: Backend, F, K> TowerRateLimiter<S, T, B, F, K> {
    pub fn new(inner: S, limiter: RateLimiter<T, B>, callback: F, key_extractor: K) -> Self {
        TowerRateLimiter {
            inner,
            limiter,
            callback,
            key_extractor,
        }
    }
}

impl<S, ReqBody, ResBody, F, T, B, K> Service<Request<ReqBody>> for TowerRateLimiter<S, T, B, F, K>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    T: LimiterType,
    B: Backend,
    ResBody: Default,
    F: Fn(Request<ReqBody>) -> Response<ResBody>,
    K: Fn(&Request<ReqBody>) -> String,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Either<Ready<Result<Self::Response, Self::Error>>, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        if self
            .limiter
            .is_ratelimited(&(self.key_extractor)(&request))
            .is_err()
        {
            let response = (self.callback)(request);
            Either::Left(ready(Ok(response)))
        } else {
            Either::Right(self.inner.call(request))
        }
    }
}

#[derive(Debug, Clone)]
pub struct TowerRateLimiterLayer<T, B, F, K> {
    limiter: RateLimiter<T, B>,
    callback: F,
    key_extractor: K,
}

impl<T: LimiterType, B: Backend, F, K> TowerRateLimiterLayer<T, B, F, K> {
    pub fn new(limiter: RateLimiter<T, B>, callback: F, key_extractor: K) -> Self {
        TowerRateLimiterLayer {
            limiter,
            callback,
            key_extractor,
        }
    }
}

pub fn default_callback<T, S: Default>(_: Request<T>) -> Response<S> {
    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body(S::default())
        .unwrap()
}

impl<T: LimiterType, B: Backend, ReqBody, ResBody: Default, K>
    TowerRateLimiterLayer<T, B, fn(Request<ReqBody>) -> Response<ResBody>, K>
{
    pub fn default(limiter: RateLimiter<T, B>, key_extractor: K) -> Self {
        TowerRateLimiterLayer {
            limiter,
            callback: default_callback,
            key_extractor,
        }
    }
}

impl<S, T, B, F: Clone, K: Clone> Layer<S> for TowerRateLimiterLayer<T, B, F, K>
where
    T: LimiterType,
    B: Backend,
{
    type Service = TowerRateLimiter<S, T, B, F, K>;

    fn layer(&self, service: S) -> Self::Service {
        TowerRateLimiter::new(
            service,
            self.limiter.clone(),
            self.callback.clone(),
            self.key_extractor.clone(),
        )
    }
}
