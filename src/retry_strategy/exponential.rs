use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

#[derive(Debug, Copy, Clone, Default)]
pub struct ExponentialRetryStrategy {
    pub max_attempts: usize,
    pub initial_delay: Duration,
}

impl ExponentialRetryStrategy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn initial_delay(mut self, initial_delay: Duration) -> Self {
        self.initial_delay = initial_delay;
        self
    }
}

impl RetryStrategy for ExponentialRetryStrategy {
    fn check_attempt(&mut self, attempts_before: usize) -> Result<Duration, TooManyAttempts> {
        let exponent = 2_usize.pow(attempts_before as u32);
        if self.max_attempts == attempts_before {
            Err(TooManyAttempts)
        } else {
            Ok(self.initial_delay * exponent as u32)
        }
    }
}
