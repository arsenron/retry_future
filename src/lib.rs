pub mod error;
mod future;
mod retry_strategy;

pub use error::{Error, RetryError, TooManyAttempts};
pub use future::RetryFuture;
pub use retry_strategy::{
    ExponentialRetryStrategy, InfiniteRetryStrategy, LinearRetryStrategy, RetryStrategy,
};
use std::fmt::Debug;

/// Return type of [inner future](crate::future::FutureFactory::Future)
/// inside [RetryFuture](crate::future::RetryFuture)
///
/// `Fail` variant means unrecoverable error
///
/// If `future` propagates errors early by using `?` then
/// `Retry` will contain [error](error::Error) inside it.
///
/// If you want to provide some debug information about
/// why a `Future` failed, you can construct [error](error::Error) youself.
#[derive(Debug)]
pub enum RetryPolicy<E = String> {
    Retry(Option<Error>),
    /// Unrecoverable error which means that the [RetryFuture](crate::future::RetryFuture)
    /// `Future` will immediately return with an error
    Fail(E),
}

impl<E, T: Into<anyhow::Error>> From<T> for RetryPolicy<E> {
    fn from(t: T) -> Self {
        Self::Retry(Some(Error { error: t.into(), is_early_returned: true }))
    }
}

/// Return early with [RetryPolicy::Fail](crate::RetryPolicy::Fail)
#[macro_export]
macro_rules! fail {
    ($e:expr) => {
        return Err($crate::RetryPolicy::Fail($e))
    };
}

/// Return early with [RetryPolicy::Retry](crate::RetryPolicy::Retry)
#[macro_export]
macro_rules! retry {
    ($e:expr) => {
        return Err($crate::RetryPolicy::Retry(Some($crate::error::Error::msg($e))))
    };

    () => {
        return Err($crate::RetryPolicy::Retry(None))
    };
}

#[cfg(test)]
mod tests {
    pub use super::*;
    use futures::{
        future::{err, ok},
        TryFutureExt,
    };
    use std::time::Duration;

    struct MyRetryStrategy {
        max_attempts: usize,
        counter: Vec<u8>,
    }

    impl RetryStrategy for MyRetryStrategy {
        fn check_attempt(&mut self, attempts_before: usize) -> Result<Duration, TooManyAttempts> {
            if self.max_attempts == attempts_before {
                Err(TooManyAttempts)
            } else {
                self.counter.push(0);
                Ok(Duration::from_millis(1))
            }
        }

        fn retry_early_returned_errors(&self) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn test_ok() {
        let f = RetryFuture::new(
            || ok::<_, u8>(255).map_err(|_| RetryPolicy::Fail("fail!")),
            MyRetryStrategy { max_attempts: 5, counter: vec![] },
        );
        assert_eq!(255, f.await.unwrap());
    }

    #[tokio::test]
    async fn test_fail() {
        let f = RetryFuture::new(
            || err::<u8, _>(RetryPolicy::Fail("fail")),
            MyRetryStrategy { max_attempts: 2, counter: vec![] },
        );
        let RetryPolicy::Fail(_) = f.await.unwrap_err().errors.last().unwrap() else {
            panic!("Fail error must be returned")
        };
    }

    #[tokio::test]
    async fn test_number_of_attempts() {
        let mut retry_strategy = MyRetryStrategy { max_attempts: 7, counter: vec![] };
        assert_eq!(0, retry_strategy.counter.len());
        let f = RetryFuture::new(
            || err::<u8, RetryPolicy>(RetryPolicy::Retry(None)),
            &mut retry_strategy,
        );
        f.await.unwrap_err();
        assert_eq!(7, retry_strategy.counter.len())
    }

    #[tokio::test]
    async fn test_return_early() {
        let mut retry_strategy = MyRetryStrategy { max_attempts: 7, counter: vec![] };
        let f = RetryFuture::new(
            || async {
                Ok::<_, RetryPolicy>("non-integer".parse::<u32>()?)
            },
            &mut retry_strategy,
        );
        f.await.unwrap_err();
        assert_eq!(0, retry_strategy.counter.len())
    }
}
