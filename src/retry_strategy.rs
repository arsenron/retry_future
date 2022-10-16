pub mod exponential;
pub mod infinite;
pub mod linear;

use std::time::Duration;

use crate::error::TooManyAttempts;
pub use exponential::ExponentialRetryStrategy;
pub use infinite::InfiniteRetryStrategy;
pub use linear::LinearRetryStrategy;

/// Configuration trait for [RetryFuture](crate::RetryFuture).
///
/// Goal of the trait is to return either a [duration](std::time::Duration)
/// which means how long a future needs to sleep before trying to resolve again
/// or an [error](TooManyAttempts) if there were already too many attempts.
pub trait RetryStrategy {
    /// `attempts_before` means how many attempts a [future](crate::future::FutureFactory::Future)
    /// was trying to resolve to `Ok(_)` after returning `Err(_)`.
    fn check_attempt(&mut self, attempts_before: usize) -> Result<Duration, TooManyAttempts>;

    fn retry_early_returned_errors(&self) -> bool {
        true
    }
}

impl<T> RetryStrategy for &mut T
where
    T: RetryStrategy,
{
    fn check_attempt(&mut self, attempts_before: usize) -> Result<Duration, TooManyAttempts> {
        (*self).check_attempt(attempts_before)
    }
}
