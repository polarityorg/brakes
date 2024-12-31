use brakes::{
    backend::local::Memory,
    types::{token_bucket::TokenBucket, RateLimiterError},
};

#[test]
fn invalid_cache_discard() {
    use brakes::{types::fixed_window::FixedWindow, RateLimiter};
    use std::time::Duration;

    let key = "key";
    let backend = Memory::new();

    let fixed_window_limiter = RateLimiter::builder()
        .with_backend(backend.clone())
        .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
        .build();

    let token_limiter = RateLimiter::builder()
        .with_backend(backend)
        .with_limiter(TokenBucket::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
        .with_discard_invalid_cache_entries(true)
        .build();

    assert!(fixed_window_limiter.is_ratelimited(key).is_ok());
    assert!(token_limiter.is_ratelimited(key).is_ok());
}

#[test]
fn invalid_cache_default() {
    use brakes::{types::fixed_window::FixedWindow, RateLimiter};
    use std::time::Duration;

    let key = "key";
    let backend = Memory::new();

    let fixed_window_limiter = RateLimiter::builder()
        .with_backend(backend.clone())
        .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
        .build();

    let token_limiter = RateLimiter::builder()
        .with_backend(backend)
        .with_limiter(TokenBucket::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
        .build();

    assert!(fixed_window_limiter.is_ratelimited(key).is_ok());
    assert!(token_limiter.is_ratelimited(key).is_ok());
}

#[test]
fn invalid_cache_no_discard() {
    use brakes::{types::fixed_window::FixedWindow, RateLimiter};
    use std::time::Duration;

    let key = "key";
    let backend = Memory::new();

    let fixed_window_limiter = RateLimiter::builder()
        .with_backend(backend.clone())
        .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
        .build();

    let token_limiter = RateLimiter::builder()
        .with_backend(backend)
        .with_limiter(TokenBucket::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
        .with_discard_invalid_cache_entries(false)
        .build();

    assert!(fixed_window_limiter.is_ratelimited(key).is_ok());

    let res = token_limiter.is_ratelimited(key);
    assert!(res.is_err());
    assert!(matches!(
        res,
        Err(RateLimiterError::WrongLimiterInstanceType)
    ));
}
