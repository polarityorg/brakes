use super::{LimiterInstance, LimiterType, RateLimiterError, SerializableInstance};
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct LeakyBucket {
    capacity: u32,
    leak_frequency: Duration, // leak 1 request per leak_frequency
}

impl LeakyBucket {
    pub fn new(capacity: u32, leak_frequency: Duration) -> Self {
        LeakyBucket {
            capacity,
            leak_frequency,
        }
    }
}

impl LimiterType for LeakyBucket {
    fn is_ratelimited(&self, bytes: Option<Vec<u8>>) -> Result<Vec<u8>, RateLimiterError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mut instance = match bytes {
            Some(b) => self.window_instance(b)?.as_leaky_bucket_instance()?,
            None => LeakyBucketInstance {
                processed: 0,
                last_leaked: now,
            },
        };

        let elapsed = now - instance.last_leaked();

        instance.processed -= cmp::min(
            (elapsed as f64 / self.leak_frequency.as_millis() as f64).floor() as u32,
            instance.processed,
        );
        instance.last_leaked = now;

        if instance.processed >= self.capacity {
            return Err(RateLimiterError::RateExceeded);
        }
        instance.processed += 1;
        instance.to_bytes()
    }

    fn window_instance(&self, value: Vec<u8>) -> Result<LimiterInstance, RateLimiterError> {
        Ok(LimiterInstance::LeakyBucketInstance(
            LeakyBucketInstance::from_bytes(value)?,
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeakyBucketInstance {
    processed: u32,
    last_leaked: u128,
}

impl LeakyBucketInstance {
    pub fn new(last_leaked: u128, processed: u32) -> Self {
        LeakyBucketInstance {
            last_leaked,
            processed,
        }
    }
    pub fn processed(&self) -> u32 {
        self.processed
    }

    pub fn last_leaked(&self) -> u128 {
        self.last_leaked
    }
}

impl SerializableInstance for LeakyBucketInstance {}
