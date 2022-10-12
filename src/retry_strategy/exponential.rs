use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

pub struct ExponentialRetryStrategy {
    pub max_attempts: usize,
    pub starts_with: Duration,
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
