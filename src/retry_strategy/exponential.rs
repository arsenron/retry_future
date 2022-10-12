use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

#[derive(Debug, Copy, Clone, Default)]
pub struct ExponentialRetryStrategy {
    pub max_attempts: usize,
    pub starts_with: Duration,
}

impl ExponentialRetryStrategy {
    pub fn max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn starts_with(mut self, starts_with: Duration) -> Self {
        self.starts_with = starts_with;
        self
    }
}

impl RetryStrategy for ExponentialRetryStrategy {
    fn check_attempt(&mut self, attempts_before: usize) -> Result<Duration, TooManyAttempts> {
        let exponent = 2_usize.pow(attempts_before as u32);
        if self.max_attempts == attempts_before {
            Err(TooManyAttempts)
        } else {
            Ok(self.starts_with * exponent as u32)
        }
    }
}
