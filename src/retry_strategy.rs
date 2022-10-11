pub mod exponential;
pub mod infinite;
pub mod linear;

use crate::error::TimeoutError;
pub use exponential::ExponentialRetryStrategy;
pub use infinite::InfiniteRetryStrategy;
pub use linear::LinearRetryStrategy;
use std::time::Duration;

pub trait RetryStrategy {
    fn check_attempt(&mut self, current_attempt: usize) -> Result<Duration, TimeoutError>;
}
