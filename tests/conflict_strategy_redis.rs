#[test]
#[cfg(feature = "redis")]
fn retry_and_deny() {
    use brakes::{backend::redis::RedisBackend, types::fixed_window::FixedWindow, RateLimiter};
    use redis::Commands;
    use std::{thread, time::Duration};

    let key = "key";

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .connection_timeout(Duration::from_secs(1))
        .build(client)
        .unwrap();

    pool.get().unwrap().del::<&str, ()>(key).unwrap();

    let limiter = RateLimiter::builder()
        .with_backend(RedisBackend::new(pool))
        .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(1))
        .build();

    let mut threads = vec![];
    for _ in 0..3 {
        let limiter = limiter.clone();
        threads.push(thread::spawn(move || limiter.is_ratelimited(key)));
    }
    let (mut ok, mut err) = (0, 0);
    for t in threads {
        if t.join().unwrap().is_ok() {
            ok += 1;
        } else {
            err += 1;
        }
    }
    assert_eq!(ok, 2);
    assert_eq!(err, 1);
}

#[test]
#[cfg(feature = "redis")]
fn retry_and_allow() {
    use brakes::{backend::redis::RedisBackend, types::fixed_window::FixedWindow, RateLimiter};
    use redis::Commands;
    use std::{thread, time::Duration};

    let key = "key";

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .connection_timeout(Duration::from_secs(1))
        .build(client)
        .unwrap();

    pool.get().unwrap().del::<&str, ()>(key).unwrap();

    let limiter = RateLimiter::builder()
        .with_backend(RedisBackend::new(pool))
        .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndAllow(1))
        .build();

    let mut threads = vec![];
    for _ in 0..5 {
        let limiter = limiter.clone();
        threads.push(thread::spawn(move || limiter.is_ratelimited("key2")));
    }
    for t in threads {
        assert!(t.join().unwrap().is_ok());
    }
}

#[test]
#[cfg(feature = "redis")]
fn deny() {
    use brakes::{backend::redis::RedisBackend, types::fixed_window::FixedWindow, RateLimiter};
    use redis::Commands;
    use std::{thread, time::Duration};

    let key = "key";

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .connection_timeout(Duration::from_secs(1))
        .build(client)
        .unwrap();

    pool.get().unwrap().del::<&str, ()>(key).unwrap();

    let limiter = RateLimiter::builder()
        .with_backend(RedisBackend::new(pool))
        .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::Deny)
        .build();

    let mut threads = vec![];
    for _ in 0..3 {
        let limiter = limiter.clone();
        threads.push(thread::spawn(move || limiter.is_ratelimited("key3")));
    }
    let (mut ok, mut err) = (0, 0);
    for t in threads {
        if t.join().unwrap().is_ok() {
            ok += 1;
        } else {
            err += 1;
        }
    }
    assert!(ok == 1);
    assert!(err == 2);
}

#[test]
#[cfg(feature = "redis")]
fn allow() {
    use brakes::{backend::redis::RedisBackend, types::fixed_window::FixedWindow, RateLimiter};
    use redis::Commands;
    use std::{thread, time::Duration};

    let key = "key";

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .connection_timeout(Duration::from_secs(1))
        .build(client)
        .unwrap();

    pool.get().unwrap().del::<&str, ()>(key).unwrap();

    let limiter = RateLimiter::builder()
        .with_backend(RedisBackend::new(pool))
        .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::Allow)
        .build();

    let mut threads = vec![];
    for _ in 0..5 {
        let limiter = limiter.clone();
        threads.push(thread::spawn(move || limiter.is_ratelimited("key4")));
    }
    for t in threads {
        assert!(t.join().unwrap().is_ok());
    }
}
