use super::{fixed_window::FixedWindowInstance, LimiterInstance, LimiterType, RateLimiterError};
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct SlidingWindowCounter {
    threshold: u32,
    window_length: Duration,
}

impl SlidingWindowCounter {
    pub fn new(threshold: u32, window_length: Duration) -> Self {
        SlidingWindowCounter {
            threshold,
            window_length,
        }
    }
}

impl LimiterType for SlidingWindowCounter {
    fn is_ratelimited(&self, bytes: Option<Vec<u8>>) -> Result<LimiterInstance, RateLimiterError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mut instance = match bytes {
            Some(b) => self.window_instance(b)?.as_sliding_window_instance()?,
            None => SlidingWindowInstance {
                current: FixedWindowInstance::new(now, 0),
                previous: FixedWindowInstance::new(now, 0),
            },
        };

        if instance.current.window_start() + self.window_length.as_millis() < now {
            instance.previous = instance.current;
            instance.current = FixedWindowInstance::new(now, 0)
        }

        let start = cmp::max(0, now - self.window_length.as_millis());
        let prev_end = instance.previous.window_start() + self.window_length.as_millis();
        let weight: f64 = cmp::max(0, prev_end as i64 - start as i64) as f64
            / self.window_length.as_millis() as f64;
        let count = (instance.previous.window_count() as f64 * weight)
            + instance.current.window_count() as f64;
        if count >= self.threshold as f64 {
            return Err(RateLimiterError::RateExceeded);
        }

        instance.current.count += 1;
        Ok(LimiterInstance::SlidingWindowInstance(instance))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlidingWindowInstance {
    current: FixedWindowInstance,
    previous: FixedWindowInstance,
}

impl SlidingWindowInstance {
    pub fn current_window(&self) -> &FixedWindowInstance {
        &self.current
    }

    pub fn previous_window(&self) -> &FixedWindowInstance {
        &self.previous
    }
}
