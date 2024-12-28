use super::{LimiterInstance, LimiterType, RateLimiterError, SerializableInstance};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u32,
    fill_frequency: Duration, // 1 token per fill_frequency
}

impl TokenBucket {
    pub fn new(capacity: u32, fill_frequency: Duration) -> Self {
        TokenBucket {
            capacity,
            fill_frequency,
        }
    }
}

impl LimiterType for TokenBucket {
    fn is_ratelimited(&self, bytes: Option<Vec<u8>>) -> Result<Vec<u8>, RateLimiterError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mut instance = match bytes {
            Some(b) => self.window_instance(b)?.as_token_bucket_instance()?,
            None => TokenBucketInstance {
                tokens: self.capacity as f32,
                last_access: now,
            },
        };

        let elapsed = now - instance.last_access();

        instance.tokens = instance.tokens + elapsed as f32 / self.fill_frequency.as_millis() as f32;
        if instance.tokens > self.capacity as f32 {
            instance.tokens = self.capacity as f32;
        }

        if instance.tokens < 1f32 {
            return Err(RateLimiterError::RateExceeded);
        }
        instance.tokens -= 1f32;
        instance.last_access = now;
        instance.to_bytes()
    }

    fn window_instance(&self, value: Vec<u8>) -> Result<LimiterInstance, RateLimiterError> {
        Ok(LimiterInstance::TokenBucketInstance(
            TokenBucketInstance::from_bytes(value)?,
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenBucketInstance {
    tokens: f32,
    last_access: u128,
}

impl TokenBucketInstance {
    pub fn new(last_access: u128, tokens: f32) -> Self {
        TokenBucketInstance {
            last_access,
            tokens,
        }
    }
    pub fn tokens(&self) -> f32 {
        self.tokens
    }

    pub fn last_access(&self) -> u128 {
        self.last_access
    }
}

impl SerializableInstance for TokenBucketInstance {}
