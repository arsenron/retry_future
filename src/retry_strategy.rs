pub mod exponential;
pub mod infinite;
pub mod linear;

use std::time::Duration;

use crate::error::TimeoutError;
pub use exponential::ExponentialRetryStrategy;
pub use infinite::InfiniteRetryStrategy;
pub use linear::LinearRetryStrategy;

/// Used as the second argument for [FutureRetry](crate::FutureRetry::new)
/// constructor.
///
/// The main purpose of the trait is to return either a [duration](std::time::Duration)
/// which means how long a future needs to sleep before trying to resolve again
/// or an [error](TimeoutError) if there were already too many attempts.
pub trait RetryStrategy {
    /// `current_attempt` represents attempt number for a future.
    ///
    /// For example, if `current_attempt` equals 5,
    /// a future was attempted to resolve without success 4 times before
    fn check_attempt(&mut self, current_attempt: usize) -> Result<Duration, TimeoutError>;
}
