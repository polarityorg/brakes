use super::{Backend, BackendError};

#[derive(Clone)]
pub struct MemCache {
    client: memcache::Client,
}

impl MemCache {
    pub fn new(client: memcache::Client) -> Self {
        MemCache { client }
    }
}

impl Backend for MemCache {
    fn get(&self, key: &str) -> Result<(Vec<u8>, Option<u64>), BackendError> {
        let (v, _, cas) = match self.client.get(&key) {
            Ok(Some(v)) => v,
            Ok(None) => return Err(BackendError::KeyMissing),
            Err(e) => return Err(BackendError::MemCacheError(e)),
        };
        Ok((v, cas))
    }

    fn set(&self, key: &str, value: &Vec<u8>, cas: Option<u64>) -> Result<(), BackendError> {
        match cas {
            Some(cas) => match self.client.cas(&key, &value[..], u32::MAX, cas) {
                Ok(false) => return Err(BackendError::ValueChanged),
                Err(e) => return Err(BackendError::MemCacheError(e)),
                Ok(true) => Ok(()),
            },
            None => self
                .client
                .set(&key, &value[..], u32::MAX)
                .map_err(|e| BackendError::MemCacheError(e)),
        }
    }

    fn delete(&self, key: &str) -> Result<(), BackendError> {
        match self.client.delete(&key) {
            Ok(_) => Ok(()),
            Err(e) => Err(BackendError::MemCacheError(e)),
        }
    }
}
