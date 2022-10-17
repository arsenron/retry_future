use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

/// Simple retry strategy that is retrying futures after [Duration](std::time::Duration)
pub struct LinearRetryStrategy {
    pub max_attempts: usize,
    pub delay_between_retries: Duration,
    /// See [RetryStrategy::retry_early_returned_errors](crate::retry_strategy::RetryStrategy::retry_early_returned_errors)
    pub retry_early_returned_errors: bool,
}

impl Default for LinearRetryStrategy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            delay_between_retries: Duration::from_millis(500),
            retry_early_returned_errors: true,
        }
    }
}

impl RetryStrategy for LinearRetryStrategy {
    fn check_attempt(&mut self, attempts_before: usize) -> Result<Duration, TooManyAttempts> {
        if self.max_attempts == attempts_before {
            Err(TooManyAttempts)
        } else {
            Ok(self.delay_between_retries)
        }
    }

    fn retry_early_returned_errors(&self) -> bool {
        self.retry_early_returned_errors
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

    pub fn delay_between_retries(mut self, delay_between_retries: Duration) -> Self {
        self.delay_between_retries = delay_between_retries;
        self
    }

    /// See [RetryStrategy::retry_early_returned_errors](crate::retry_strategy::RetryStrategy::retry_early_returned_errors)
    pub fn retry_early_returned_errors(mut self, retry_early_returned_errors: bool) -> Self {
        self.retry_early_returned_errors = retry_early_returned_errors;
        self
    }
}
