use crate::RetryPolicy;
use std::fmt::{Debug, Display, Formatter};

/// Error returned from [AsyncRetry](crate::AsyncRetry::poll), i.e.
/// when we await [AsyncRetry](crate::AsyncRetry), the returned type is `Result<T, RetryError<E>>`
///
/// This type accumulates all errors that happen inside inner future.
/// That means that after a future failed to resolve to Ok(_),
/// then an error is pushed to `errors` Vec
pub struct RetryError<E> {
    pub errors: Vec<RetryPolicy<E>>,
}

impl<E: Display> Display for RetryError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, e) in self.errors.iter().enumerate() {
            match e {
                RetryPolicy::Repeat(maybe_error) => {
                    writeln!(f, "{}", "-".repeat(100))?;
                    writeln!(f, "Attempt {i} ")?;
                    writeln!(f, "TooManyRepeats: {maybe_error:?}")?;
                }
                RetryPolicy::Fail(fail) => writeln!(f, "Fail: {fail}")?,
            }
        }
        Ok(())
    }
}

impl<E: Display> Debug for RetryError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<E: Display> std::error::Error for RetryError<E> {}

/// Type to be used in [RetryStrategy](crate::retry_strategy::RetryStrategy)
#[derive(Debug, Copy, Clone)]
pub struct TooManyAttempts;

impl Display for TooManyAttempts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl std::error::Error for TooManyAttempts {}
