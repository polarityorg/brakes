use super::{Backend, BackendError};
use redis::{cmd, pipe, Commands};

#[derive(Clone)]
pub struct RedisClusterBackend {
    pool: r2d2::Pool<redis::cluster::ClusterClient>,
}

impl RedisClusterBackend {
    pub fn new(pool: r2d2::Pool<redis::cluster::ClusterClient>) -> Self {
        Self { pool }
    }
}

impl Backend for RedisClusterBackend {
    fn get(&self, key: &str) -> Result<(Vec<u8>, Option<u64>), BackendError> {
        match self.pool.get() {
            Ok(mut conn) => {
                if let Err(e) = cmd("WATCH").arg(&[key]).exec(&mut conn) {
                    return Err(BackendError::RedisError(e));
                }
                match conn.get::<&str, Option<Vec<u8>>>(key) {
                    Ok(Some(v)) => Ok((v, None)),
                    Ok(None) => Err(BackendError::KeyMissing),
                    Err(e) => Err(BackendError::RedisError(e)),
                }
            }
            Err(e) => Err(BackendError::R2D2Error(e)),
        }
    }

    fn set(&self, key: &str, value: &[u8], _: Option<u64>) -> Result<(), BackendError> {
        match self.pool.get() {
            Ok(mut conn) => {
                let mut pipe = pipe();
                let result = match pipe
                    .atomic()
                    .set(key, value)
                    .ignore()
                    .query::<Option<()>>(&mut conn)
                {
                    Ok(Some(_)) => Ok(()),
                    Ok(None) => Err(BackendError::ValueChanged),
                    Err(e) => Err(BackendError::RedisError(e)),
                };
                if let Err(e) = cmd("UNWATCH").exec(&mut conn) {
                    return Err(BackendError::RedisError(e));
                }
                result
            }
            Err(e) => Err(BackendError::R2D2Error(e)),
        }
    }

    fn delete(&self, key: &str) -> Result<(), BackendError> {
        match self.pool.get() {
            Ok(mut conn) => match conn.del::<&str, ()>(key) {
                Ok(_) => Ok(()),
                Err(e) => Err(BackendError::RedisError(e)),
            },
            Err(e) => Err(BackendError::R2D2Error(e)),
        }
    }
}
