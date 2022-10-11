use crate::{RetryStrategy, TimeoutError};
use std::time::Duration;

pub struct ExponentialRetryStrategy {
    pub attempts: usize,
    pub start_with: Duration,
}

impl RetryStrategy for ExponentialRetryStrategy {
    fn check_attempt(&mut self, current_attempt: usize) -> Result<Duration, TimeoutError> {
        if self.attempts == current_attempt {
            Err(TimeoutError)
        } else {
            Ok(self.start_with * current_attempt as u32)
        }
    }
}
