#[test]
#[cfg(feature = "memcache")]
fn retry_and_deny() {
    use brakes::{backend::memcache::MemCache, types::fixed_window::FixedWindow, RateLimiter};
    use std::{thread, time::Duration};

    let key = "key";

    let cache = memcache::connect("memcache://127.0.0.1:11211").unwrap();
    let _ = cache.delete(key);

    let limiter = RateLimiter::builder()
        .with_backend(MemCache::new(cache))
        .with_limiter(FixedWindow::new(10000, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::RetryAndDeny(2))
        .build();

    let mut threads = vec![];
    for _ in 0..10 {
        let limiter = limiter.clone();
        threads.push(thread::spawn(move || limiter.is_ratelimited("key1")));
    }
    let (mut ok, mut err) = (0, 0);
    for t in threads {
        if t.join().unwrap().is_ok() {
            ok += 1;
        } else {
            err += 1;
        }
    }
    assert!(ok > 5);
    assert!(err < 5);
}

#[test]
#[cfg(feature = "memcache")]
fn retry_and_allow() {
    use brakes::{backend::memcache::MemCache, types::fixed_window::FixedWindow, RateLimiter};
    use std::{thread, time::Duration};

    let key = "key";

    let cache = memcache::connect("memcache://127.0.0.1:11211").unwrap();
    let _ = cache.delete(key);

    let limiter = RateLimiter::builder()
        .with_backend(MemCache::new(cache))
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
#[cfg(feature = "memcache")]
fn deny() {
    use brakes::{backend::memcache::MemCache, types::fixed_window::FixedWindow, RateLimiter};
    use std::{thread, time::Duration};

    let key = "key";

    let cache = memcache::connect("memcache://127.0.0.1:11211").unwrap();
    let _ = cache.delete("key");

    let limiter = RateLimiter::builder()
        .with_backend(MemCache::new(cache))
        .with_limiter(FixedWindow::new(100, Duration::from_millis(1000)))
        .with_conflict_strategy(brakes::RetryStrategy::Deny)
        .build();

    let mut threads = vec![];
    for _ in 0..10 {
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
    assert!(ok > 0 && ok < 10);
    assert!(err > 0 && err < 10);
}

#[test]
#[cfg(feature = "memcache")]
fn allow() {
    use brakes::{backend::memcache::MemCache, types::fixed_window::FixedWindow, RateLimiter};
    use std::{thread, time::Duration};

    let key = "key";

    let cache = memcache::connect("memcache://127.0.0.1:11211").unwrap();
    let _ = cache.delete(key);

    let limiter = RateLimiter::builder()
        .with_backend(MemCache::new(cache))
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
