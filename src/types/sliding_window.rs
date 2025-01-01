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
        self.is_rate_limited_now(now, bytes)
    }
}

impl SlidingWindowCounter {
    fn is_rate_limited_now(
        &self,
        now: u128,
        bytes: Option<Vec<u8>>,
    ) -> Result<LimiterInstance, RateLimiterError> {
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

#[test]
fn sliding_window_counter() {
    use crate::types::SerializableInstance;

    let mut instance = None;
    let counter = SlidingWindowCounter::new(5, Duration::from_millis(100));
    let mut ts = 1000u128;

    for _ in 0..5 {
        let result = counter.is_rate_limited_now(ts, instance);
        assert!(result.is_ok());
        instance = Some(result.unwrap().to_bytes().unwrap());
        ts += 20;
    }

    // should fall within the same window, should fail
    let result = counter.is_rate_limited_now(ts, instance.clone());
    assert!(result.is_err());

    // should only allow 1 request (20%)
    ts += 20;
    for i in 0..2 {
        let result = counter.is_rate_limited_now(ts, instance.clone());
        assert!(result.is_ok() == (i < 1));
        instance = match result {
            Ok(i) => Some(i.to_bytes().unwrap()),
            Err(_) => instance.clone(),
        }
    }

    // new window should accept only 5 concurrent requests
    ts += 101;
    for i in 0..6 {
        let result = counter.is_rate_limited_now(ts, instance.clone());
        assert!(result.is_ok() == (i < 5));
        instance = match result {
            Ok(i) => Some(i.to_bytes().unwrap()),
            Err(_) => instance.clone(),
        };
    }
}
