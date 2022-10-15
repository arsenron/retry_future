use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

/// Simple retry strategy that is retrying futures after [Duration](std::time::Duration)
pub struct LinearRetryStrategy {
    pub max_attempts: usize,
    pub delay_between_repeats: Duration,
}

impl Default for LinearRetryStrategy {
    fn default() -> Self {
        Self { max_attempts: 5, delay_between_repeats: Duration::from_millis(500) }
    }
}

impl RetryStrategy for LinearRetryStrategy {
    fn check_attempt(&mut self, attempts_before: usize) -> Result<Duration, TooManyAttempts> {
        if self.max_attempts == attempts_before {
            Err(TooManyAttempts)
        } else {
            Ok(self.delay_between_repeats)
        }
    }
}

impl LinearRetryStrategy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn delay_between_repeats(mut self, delay_between_repeats: Duration) -> Self {
        self.delay_between_repeats = delay_between_repeats;
        self
    }
}
