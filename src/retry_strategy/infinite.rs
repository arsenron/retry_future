use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

/// Infinite retry
pub struct InfiniteRetryStrategy {
    pub duration_between_retries: Duration,
}

impl RetryStrategy for InfiniteRetryStrategy {
    fn check_attempt(&mut self, _attempts_before: usize) -> Result<Duration, TooManyAttempts> {
        Ok(self.duration_between_retries)
    }

    fn retry_early_returned_errors(&self) -> bool {
        true
    }
}
