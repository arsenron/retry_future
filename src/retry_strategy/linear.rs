use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

pub struct LinearRetryStrategy {
    max_attempts: usize,
    duration_between_repeats: Duration,
}

impl Default for LinearRetryStrategy {
    fn default() -> Self {
        Self { max_attempts: 5, duration_between_repeats: Duration::from_millis(500) }
    }
}

impl RetryStrategy for LinearRetryStrategy {
    fn check_attempt(&mut self, attempts_before: usize) -> Result<Duration, TooManyAttempts> {
        if self.max_attempts == attempts_before {
            Err(TooManyAttempts)
        } else {
            Ok(self.duration_between_repeats)
        }
    }
}

impl LinearRetryStrategy {
    pub fn max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn duration_between_repeats(mut self, duration_between_repeats: Duration) -> Self {
        self.duration_between_repeats = duration_between_repeats;
        self
    }
}
