use crate::{backend::Backend, types::LimiterType, RateLimiter};
use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpRequest, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

#[derive(Clone)]
pub struct ActixwebRateLimiter<T, B> {
    limiter: RateLimiter<T, B>,
    callback: fn(&HttpRequest) -> HttpResponse,
    key_extractor: fn(&HttpRequest) -> String,
}

impl<T: LimiterType, B: Backend> ActixwebRateLimiter<T, B> {
    pub fn new(limiter: RateLimiter<T, B>) -> Self {
        let default_callback = |_: &HttpRequest| HttpResponse::TooManyRequests().finish();
        let default_extractor = |req: &HttpRequest| req.peer_addr().unwrap().ip().to_string();

        ActixwebRateLimiter {
            limiter,
            callback: default_callback,
            key_extractor: default_extractor,
        }
    }

    pub fn with_callback(mut self, callback: fn(&HttpRequest) -> HttpResponse) -> Self {
        self.callback = callback;
        self
    }

    pub fn with_key_extractor(mut self, extractor: fn(&HttpRequest) -> String) -> Self {
        self.key_extractor = extractor;
        self
    }
}

impl<S, B, LT, BE> Transform<S, ServiceRequest> for ActixwebRateLimiter<LT, BE>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + From<BoxBody> + 'static,
    LT: LimiterType,
    BE: Backend,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ActixwebRateLimiterMiddleware<S, LT, BE>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ActixwebRateLimiterMiddleware {
            service,
            limiter: self.limiter.clone(),
            callback: self.callback,
            key_extractor: self.key_extractor,
        }))
    }
}

pub struct ActixwebRateLimiterMiddleware<S, T, B> {
    service: S,
    limiter: RateLimiter<T, B>,
    callback: fn(&HttpRequest) -> HttpResponse,
    key_extractor: fn(&HttpRequest) -> String,
}

impl<S, B, LT, BE> Service<ServiceRequest> for ActixwebRateLimiterMiddleware<S, LT, BE>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + From<BoxBody> + 'static,
    LT: LimiterType,
    BE: Backend,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if self
            .limiter
            .is_ratelimited(&(self.key_extractor)(req.request()))
            .is_err()
        {
            let response = (self.callback)(req.request());
            let service_response = req.into_response(response.map_into_boxed_body());
            return Box::pin(async { Ok(service_response.map_body(|_, body| B::from(body))) });
        }

        let res = self.service.call(req);
        Box::pin(async move { Ok(res.await?) })
    }
}
