use crate::{RetryStrategy, TimeoutError};
use std::time::Duration;

pub struct InfiniteRetryStrategy {
    pub duration_between_repeats: Duration,
}

impl RetryStrategy for InfiniteRetryStrategy {
    fn check_attempt(&mut self, _current_attempt: usize) -> Result<Duration, TimeoutError> {
        Ok(self.duration_between_repeats)
    }
}
