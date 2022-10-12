use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

pub struct InfiniteRetryStrategy {
    pub duration_between_repeats: Duration,
}

impl RetryStrategy for InfiniteRetryStrategy {
    fn check_attempt(&mut self, _attempts_before: usize) -> Result<Duration, TooManyAttempts> {
        Ok(self.duration_between_repeats)
    }
}
