use crate::{RetryStrategy, TimeoutError};
use std::time::Duration;

pub struct LinearRetryStrategy {
    pub attempts: usize,
    pub duration_between_repeats: Duration,
}

impl Default for LinearRetryStrategy {
    fn default() -> Self {
        Self { attempts: 5, duration_between_repeats: Duration::from_millis(500) }
    }
}

impl RetryStrategy for LinearRetryStrategy {
    fn check_attempt(&mut self, current_attempt: usize) -> Result<Duration, TimeoutError> {
        if self.attempts == current_attempt {
            Err(TimeoutError)
        } else {
            Ok(self.duration_between_repeats)
        }
    }
}

impl LinearRetryStrategy {
    pub fn attempts(mut self, repeat: usize) -> Self {
        self.attempts = repeat;
        self
    }

    pub fn duration_between_repeats(mut self, duration_between_repeats: Duration) -> Self {
        self.duration_between_repeats = duration_between_repeats;
        self
    }
}
