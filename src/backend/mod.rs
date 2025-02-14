pub mod local;

#[cfg(feature = "memcache")]
#[cfg_attr(docsrs, doc(cfg(feature = "memcache")))]
pub mod memcache;
#[cfg(feature = "redis")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis")))]
pub mod redis;
#[cfg(feature = "redis-cluster")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis-cluster")))]
pub mod redis_cluster;

#[cfg(feature = "memcache")]
use ::memcache::MemcacheError;
#[cfg(feature = "redis")]
use ::redis::RedisError;
use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

pub trait Backend: Clone {
    fn get(&self, key: &str) -> Result<(Vec<u8>, Option<u64>), BackendError>;
    fn set(&self, key: &str, value: &[u8], version: Option<u64>) -> Result<(), BackendError>;
    fn delete(&self, key: &str) -> Result<(), BackendError>;

    fn get_with_retries(
        &self,
        key: &str,
        tries: u32,
    ) -> Result<(Vec<u8>, Option<u64>), BackendError> {
        let mut err = None;
        for _ in 0..tries {
            match self.get(key) {
                Ok(v) => return Ok(v),
                Err(BackendError::KeyMissing) => return Err(BackendError::KeyMissing),
                Err(e) => {
                    err = Some(e);
                    continue;
                }
            }
        }
        Err(err.unwrap())
    }

    fn set_with_retries(
        &self,
        key: &str,
        value: Vec<u8>,
        version: Option<u64>,
        tries: u32,
    ) -> Result<(), BackendError> {
        let mut err = None;
        for _ in 0..tries {
            match self.set(key, &value, version) {
                Ok(_) => return Ok(()),
                Err(BackendError::ValueChanged) => return Err(BackendError::ValueChanged),
                Err(e) => {
                    err = Some(e);
                    continue;
                }
            }
        }
        Err(err.unwrap())
    }

    fn delete_with_retries(&self, key: &str, tries: u32) -> Result<(), BackendError> {
        let mut err = None;
        for _ in 0..tries {
            match self.delete(key) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    err = Some(e);
                    continue;
                }
            }
        }
        Err(err.unwrap())
    }
}

#[derive(Debug)]
pub enum BackendError {
    #[cfg(feature = "redis")]
    R2D2Error(r2d2::Error),
    #[cfg(feature = "redis")]
    RedisError(RedisError),
    #[cfg(feature = "memcache")]
    MemCacheError(MemcacheError),
    LocalMemLockError,
    KeyMissing,
    ValueChanged,
}

impl Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "redis")]
            BackendError::R2D2Error(e) => std::fmt::Display::fmt(&e, f),
            #[cfg(feature = "redis")]
            BackendError::RedisError(e) => std::fmt::Display::fmt(&e, f),
            #[cfg(feature = "memcache")]
            BackendError::MemCacheError(e) => std::fmt::Display::fmt(&e, f),
            BackendError::LocalMemLockError => write!(f, "mutex poison error"),
            BackendError::ValueChanged => write!(f, "value changed"),
            BackendError::KeyMissing => write!(f, "key missing"),
        }
    }
}

impl Error for BackendError {}
