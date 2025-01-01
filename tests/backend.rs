use brakes::backend::{local::Memory, Backend};

#[test]
fn memory() {
    let backend = Memory::new();
    test_backend(backend);
}

#[cfg(feature = "memcache")]
#[test]
fn memcache() {
    use brakes::backend::memcache::MemCache;

    let cache = memcache::connect("memcache://127.0.0.1:11211").unwrap();
    let backend = MemCache::new(cache);
    test_backend(backend);
}

#[cfg(feature = "redis")]
#[test]
fn redis() {
    use std::time::Duration;

    use brakes::backend::redis::RedisBackend;

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .connection_timeout(Duration::from_secs(1))
        .build(client)
        .unwrap();

    let backend = RedisBackend::new(pool);
    test_backend(backend);
}

fn test_backend(backend: impl Backend) {
    let key = "key";

    backend.set(key, &[], None).unwrap();

    let value = backend.get(key);
    assert!(value.is_ok());
    assert_eq!(value.unwrap().0, Vec::<u8>::new());

    assert!(backend.delete(key).is_ok());

    let value = backend.get(key);
    assert!(value.is_err());

    assert!(backend.delete(key).is_ok());
}
