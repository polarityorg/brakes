use std::{thread::sleep, time::Duration};

use brakes::{
    backend::local::Memory,
    types::{
        fixed_window::FixedWindow, leaky_bucket::LeakyBucket, sliding_window::SlidingWindowCounter,
        token_bucket::TokenBucket,
    },
    RateLimiter,
};

#[test]
fn fixed_window() {
    let limiter = RateLimiter::builder()
        .with_backend(Memory::new())
        .with_limiter(FixedWindow::new(2, Duration::from_secs(1)))
        .build();
    for i in 0..5 {
        let result = limiter.is_ratelimited("ip");
        assert!(result.is_ok() == (i < 2))
    }
    sleep(Duration::from_secs(1));
    for i in 0..5 {
        let result = limiter.is_ratelimited("ip");
        assert!(result.is_ok() == (i < 2))
    }
    let usage = limiter
        .get_usage("ip")
        .unwrap()
        .as_fixed_window_instance()
        .unwrap();
    assert_eq!(usage.window_count(), 2);
}

#[test]
fn sliding_window_counter() {
    let limiter = RateLimiter::builder()
        .with_backend(Memory::new())
        .with_limiter(SlidingWindowCounter::new(5, Duration::from_millis(100)))
        .build();

    // uniform across the first window
    for _ in 0..5 {
        let result = limiter.is_ratelimited("ip");
        sleep(Duration::from_millis(15));
        assert!(result.is_ok())
    }
}

#[test]
fn token_bucket() {
    let limiter = RateLimiter::builder()
        .with_backend(Memory::new())
        .with_limiter(TokenBucket::new(5, Duration::from_millis(100)))
        .build();
    for i in 0..6 {
        let result = limiter.is_ratelimited("ip");
        assert!(result.is_ok() == (i < 5))
    }

    // let the bucket fill with 2 tokens
    sleep(Duration::from_millis(200));

    for i in 0..5 {
        let result = limiter.is_ratelimited("ip");
        assert!(result.is_ok() == (i < 2))
    }
}

#[test]
fn leaky_bucket() {
    let limiter = RateLimiter::builder()
        .with_backend(Memory::new())
        .with_limiter(LeakyBucket::new(5, Duration::from_millis(100)))
        .build();
    for i in 0..6 {
        let result = limiter.is_ratelimited("ip");
        assert!(result.is_ok() == (i < 5))
    }

    // let the bucket leak 2 requests
    sleep(Duration::from_millis(200));

    for i in 0..5 {
        let result = limiter.is_ratelimited("ip");
        assert!(result.is_ok() == (i < 2))
    }
}
