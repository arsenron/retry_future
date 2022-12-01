use crate::RetryPolicy;
use std::fmt::{Debug, Display, Formatter};

/// Used in [retry policy](crate::RetryPolicy) inside
/// `Retry` variant
pub struct Error {
    pub error: anyhow::Error,
    pub(crate) is_early_returned: bool,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.error, f)
    }
}

impl Error {
    pub fn msg<M: Display + Debug + Send + Sync + 'static>(msg: M) -> Self {
        Self { error: anyhow::Error::msg(msg), is_early_returned: false }
    }

    pub fn new<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self { error: anyhow::Error::new(e), is_early_returned: false }
    }
}

/// Error returned from [RetryFuture](crate::RetryFuture::poll), i.e.
/// when we await [RetryFuture](crate::RetryFuture), the returned type is `Result<T, RetryError<E>>`
///
/// This type accumulates all errors that happen inside inner future.
/// This means that after a future fails to resolve to Ok(_), an error is pushed to `errors` Vec
pub struct RetryError<E> {
    pub errors: Vec<RetryPolicy<E>>,
}

impl<E> RetryError<E> {
    /// Returns the last error encountered
    pub fn last_error(&self) -> Option<&RetryPolicy<E>> {
        self.errors.last()
    }
}

impl<E: Debug> Display for RetryError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, retry_policy) in self.errors.iter().enumerate() {
            match retry_policy {
                RetryPolicy::Retry(maybe_error) => {
                    writeln!(f, "{}", "-".repeat(100))?;
                    writeln!(f, "Attempt {i} ")?;
                    writeln!(f, "TooManyRetries: {maybe_error:?}")?;
                }
                RetryPolicy::Fail(fail) => writeln!(f, "Fail: {fail:?}")?,
            }
        }
        Ok(())
    }
}

impl<E: Debug> Debug for RetryError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<E: Debug> std::error::Error for RetryError<E> {}

/// Type to be used in [RetryStrategy](crate::retry_strategy::RetryStrategy)
#[derive(Debug, Copy, Clone)]
pub struct TooManyAttempts;

impl Display for TooManyAttempts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl std::error::Error for TooManyAttempts {}
