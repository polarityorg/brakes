use super::{LimiterInstance, LimiterType, RateLimiterError};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct FixedWindow {
    threshold: u32,
    window_length: Duration,
}

impl FixedWindow {
    pub fn new(threshold: u32, window_length: Duration) -> Self {
        FixedWindow {
            threshold,
            window_length,
        }
    }
}

impl LimiterType for FixedWindow {
    fn is_ratelimited(&self, bytes: Option<Vec<u8>>) -> Result<LimiterInstance, RateLimiterError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mut instance = match bytes {
            Some(b) => self.window_instance(b)?.as_fixed_window_instance()?,
            None => FixedWindowInstance {
                window_start: now,
                count: 0,
            },
        };

        if now - instance.window_start >= self.window_length.as_millis() {
            instance.window_start = now;
            instance.count = 0;
        };
        if instance.count >= self.threshold {
            return Err(RateLimiterError::RateExceeded);
        }
        instance.count += 1;
        Ok(LimiterInstance::FixedWindowInstance(instance))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FixedWindowInstance {
    window_start: u128,
    pub(crate) count: u32,
}

impl FixedWindowInstance {
    pub(crate) fn new(window_start: u128, count: u32) -> Self {
        FixedWindowInstance {
            window_start,
            count,
        }
    }
    pub fn window_start(&self) -> u128 {
        self.window_start
    }

    pub fn window_count(&self) -> u32 {
        self.count
    }
}
