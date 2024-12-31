use super::{Backend, BackendError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Memory {
    map: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Backend for Memory {
    fn get(&self, key: &str) -> Result<(Vec<u8>, Option<u64>), BackendError> {
        match &self.map.lock() {
            Ok(m) => match m.get(key) {
                Some(v) => Ok((v.to_owned(), None)),
                None => Err(BackendError::KeyMissing),
            },
            Err(_) => return Err(BackendError::LocalMemLockError),
        }
    }

    fn set(&self, key: &str, value: &Vec<u8>, _: Option<u64>) -> Result<(), BackendError> {
        match self.map.lock() {
            Ok(mut m) => {
                m.insert(key.to_string(), value.clone());
                Ok(())
            }
            Err(_) => Err(BackendError::LocalMemLockError),
        }
    }

    fn delete(&self, key: &str) -> Result<(), BackendError> {
        match self.map.lock() {
            Ok(mut m) => {
                m.remove(key);
                Ok(())
            }
            Err(_) => Err(BackendError::LocalMemLockError),
        }
    }
}
