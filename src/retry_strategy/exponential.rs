use crate::{RetryStrategy, TooManyAttempts};
use std::time::Duration;

/// Retry futures exponentially.
///
/// ## Examples
///
/// ```rust
/// use retry_future::RetryStrategy;
/// use retry_future::ExponentialRetryStrategy;
/// use std::time::Duration;
///
/// let mut strategy = ExponentialRetryStrategy {
///    base: 3, max_attempts: 5, initial_delay: Duration::from_secs(1)
/// };
///
/// assert_eq!(strategy.check_attempt(0).unwrap(), Duration::from_secs(1));
/// assert_eq!(strategy.check_attempt(1).unwrap(), Duration::from_secs(3));
/// assert_eq!(strategy.check_attempt(2).unwrap(), Duration::from_secs(9));
/// assert_eq!(strategy.check_attempt(3).unwrap(), Duration::from_secs(27));
/// assert_eq!(strategy.check_attempt(4).unwrap(), Duration::from_secs(81));
///
/// assert!(strategy.check_attempt(5).is_err());
/// ```
#[derive(Debug, Copy, Clone)]
pub struct ExponentialRetryStrategy {
    pub base: usize,
    pub max_attempts: usize,
    pub initial_delay: Duration,
}

impl Default for ExponentialRetryStrategy {
    fn default() -> Self {
        Self { base: 2, max_attempts: 3, initial_delay: Duration::from_millis(500) }
    }
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
        let exponent = self.base.pow(attempts_before as u32);
        if self.max_attempts == attempts_before {
            Err(TooManyAttempts)
        } else {
            Ok(self.initial_delay * exponent as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_exponent() {
        let mut strategy = ExponentialRetryStrategy {
            base: 2,
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
        };
        assert_eq!(strategy.check_attempt(0).unwrap(), Duration::from_secs(1));
        assert_eq!(strategy.check_attempt(1).unwrap(), Duration::from_secs(2));
        assert_eq!(strategy.check_attempt(2).unwrap(), Duration::from_secs(4));
        assert_eq!(strategy.check_attempt(3).unwrap(), Duration::from_secs(8));
        assert_eq!(strategy.check_attempt(4).unwrap(), Duration::from_secs(16));

        assert!(strategy.check_attempt(5).is_err());
    }
}
