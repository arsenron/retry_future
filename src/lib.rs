mod error;
mod future;
mod retry_strategy;

pub use error::{RetryError, TooManyAttempts};
pub use future::{AsyncRetry, FutureFactory};
pub use retry_strategy::{
    ExponentialRetryStrategy, InfiniteRetryStrategy, LinearRetryStrategy, RetryStrategy,
};

/// Return type of [inner future](crate::future::FutureFactory::Future)
/// inside [AsyncRetry](crate::future::AsyncRetry)
///
/// `Fail` variant means unrecoverable error
///
/// If `future` propagates errors early by using `?` then
/// `Repeat` will contain [anyhow error](anyhow::Error) inside it.
///
/// If you want to provide some debug information about
/// why a `future` failed, you can construct [anyhow error](anyhow::Error)
/// yourself, such as `RetryPolicy::Repeat(Some(anyhow!("I failed here!")))`
#[derive(Debug)]
pub enum RetryPolicy<E> {
    Repeat(Option<anyhow::Error>),
    Fail(E),
}

impl<E, T: Into<anyhow::Error>> From<T> for RetryPolicy<E> {
    fn from(t: T) -> Self {
        Self::Repeat(Some(t.into()))
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    use futures::{
        future::{err, ok},
        TryFutureExt,
    };

    #[tokio::test]
    async fn test_ok() {
        let f = AsyncRetry::new(
            || ok::<_, u8>(255).map_err(|_| RetryPolicy::Fail("fail!")),
            LinearRetryStrategy::default(),
        );
        assert_eq!(255, f.await.unwrap());
    }

    #[tokio::test]
    async fn test_error() {
        let f = AsyncRetry::new(
            || err::<u8, _>(RetryPolicy::Fail("fail")),
            LinearRetryStrategy::default().max_attempts(1),
        );
        if let RetryError::Fail(_) = f.await.unwrap_err() {
            // ok
        } else {
            panic!("Fail error must be returned")
        }
    }
}
