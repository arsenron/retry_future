mod future;
mod error;
mod retry_strategy;

use futures::{ready, TryFuture};
use pin_project::pin_project;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::time::Duration;
use std::{
    future::Future,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time::sleep;

#[derive(Debug)]
pub enum RetryPolicy<E> {
    Repeat,
    Fail(E),
    Any(anyhow::Error),
}

impl<E, T: Into<anyhow::Error>> From<T> for RetryPolicy<E> {
    fn from(t: T) -> Self {
        Self::Any(t.into())
    }
}

pub enum RetryError<E> {
    TooManyRepeats(Option<anyhow::Error>),
    Fail(E),
}

impl<E: Display> Display for RetryError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::TooManyRepeats(maybe_error) => match maybe_error {
                Some(e) => write!(f, "TooManyRepeats: {e:?}"),
                None => write!(f, "TooManyRepeats"),
            },
            RetryError::Fail(fail) => {
                write!(f, "Fail: {fail}")
            }
        }
    }
}

impl<E: Display> Debug for RetryError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<E: Display> std::error::Error for RetryError<E> {}


#[derive(Debug, Copy, Clone)]
pub struct TimeoutError;

impl Display for TimeoutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl std::error::Error for TimeoutError {}

pub struct LinearRetryStrategy {
    pub attempts: usize,
    pub duration_between_repeats: Duration,
}

pub struct ExponentialRetryStrategy {
    pub attempts: usize,
    pub start_with: Duration,
}

pub struct InfiniteRetryStrategy {
    pub duration_between_repeats: Duration,
}

impl Default for LinearRetryStrategy {
    fn default() -> Self {
        Self {
            attempts: 5,
            duration_between_repeats: Duration::from_millis(500),
        }
    }
}

pub trait RetryStrategy {
    fn check_attempt(&mut self, current_attempt: usize) -> Result<Duration, TimeoutError>;
}

impl RetryStrategy for LinearRetryStrategy {
    fn check_attempt(&mut self, current_attempt: usize) -> Result<Duration, TimeoutError> {
        if self.attempts == current_attempt {
            Err(TimeoutError)
        } else {
            Ok(self.duration_between_repeats)
        }
    }
}

impl RetryStrategy for InfiniteRetryStrategy {
    fn check_attempt(&mut self, _current_attempt: usize) -> Result<Duration, TimeoutError> {
        Ok(self.duration_between_repeats)
    }
}

impl RetryStrategy for ExponentialRetryStrategy {
    fn check_attempt(&mut self, current_attempt: usize) -> Result<Duration, TimeoutError> {
        if self.attempts == current_attempt {
            Err(TimeoutError)
        } else {
            Ok(self.start_with * current_attempt as u32)
        }
    }
}

impl LinearRetryStrategy {
    pub fn attempts(mut self, repeat: usize) -> Self {
        self.attempts = repeat;
        self
    }

    pub fn duration_between_repeats(mut self, duration_between_repeats: Duration) -> Self {
        self.duration_between_repeats = duration_between_repeats;
        self
    }
}

pub trait FutureFactory<E> {
    type Future: TryFuture<Error=RetryPolicy<E>>;

    #[allow(clippy::wrong_self_convention)]
    fn new(&mut self) -> Self::Future;
}

impl<T, Fut, E> FutureFactory<E> for T
    where
        T: Unpin + FnMut() -> Fut,
        Fut: TryFuture<Error=RetryPolicy<E>>,
{
    type Future = Fut;

    fn new(&mut self) -> Fut {
        (self)()
    }
}

#[pin_project(project = FutureStateProj)]
enum FutureState<Fut, Output> {
    WaitingForFuture {
        #[pin]
        future: Fut,
    },
    TimerActive {
        #[pin]
        delay: tokio::time::Sleep,
    },
    NeedsPolling(Poll<Output>),
}

#[pin_project]
pub struct FutureRetry<F, E, RS>
    where F: FutureFactory<E>,
          RS: RetryStrategy
{
    factory: F,
    retry_strategy: RS,
    attempt: usize,
    #[pin]
    state: FutureState<F::Future, <Self as Future>::Output>,
    phantom: PhantomData<E>,
}

impl<F, E, RS> FutureRetry<F, E, RS>
    where
        F: FutureFactory<E>,
        RS: RetryStrategy
{
    pub fn new(mut factory: F, retry_strategy: RS) -> Self {
        let future = factory.new();
        Self {
            factory,
            retry_strategy,
            state: FutureState::WaitingForFuture { future },
            attempt: 0,
            phantom: Default::default(),
        }
    }
}

impl<F, E, RS> Future for FutureRetry<F, E, RS>
    where
        F: FutureFactory<E>,
        RS: RetryStrategy,
{
    type Output = Result<<<F as FutureFactory<E>>::Future as TryFuture>::Ok, RetryError<E>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            let future_retry = self.as_mut().project();
            let retry_strategy = future_retry.retry_strategy;
            let attempt = *future_retry.attempt;
            let new_state = match future_retry.state.project() {
                FutureStateProj::WaitingForFuture { future } => match ready!(future.try_poll(cx)) {
                    Ok(t) => {
                        *future_retry.attempt = 0;
                        FutureState::NeedsPolling(Poll::Ready(Ok(t)))
                    }
                    Err(err) => {
                        let mut move_to_next_state_depending_on_retry_strategy = |e| {
                            let check_attempt_result = retry_strategy.check_attempt(attempt);
                            match check_attempt_result {
                                Ok(duration) => FutureState::TimerActive {
                                    delay: sleep(duration),
                                },
                                Err(_) => FutureState::NeedsPolling(Poll::Ready(Err(
                                    RetryError::TooManyRepeats(e),
                                ))),
                            }
                        };
                        let new_state = match err {
                            RetryPolicy::Repeat => {
                                move_to_next_state_depending_on_retry_strategy(None)
                            }
                            RetryPolicy::Fail(s) => {
                                FutureState::NeedsPolling(Poll::Ready(Err(RetryError::Fail(s))))
                            }
                            RetryPolicy::Any(any) => {
                                move_to_next_state_depending_on_retry_strategy(Some(any.context(
                                    format!(
                                        "Failed after repeating {} times",
                                        *future_retry.attempt
                                    ),
                                )))
                            }
                        };
                        *future_retry.attempt += 1;
                        new_state
                    }
                },
                FutureStateProj::TimerActive { delay } => {
                    ready!(delay.poll(cx));
                    FutureState::WaitingForFuture {
                        future: future_retry.factory.new(),
                    }
                }
                FutureStateProj::NeedsPolling(poll) => {
                    // move from &mut T to original T
                    let output = std::mem::replace(poll, Poll::Pending);
                    return output;
                }
            };

            self.as_mut().project().state.set(new_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{
        future::{err, ok},
        TryFutureExt,
    };

    #[tokio::test]
    async fn test_ok() {
        let f = FutureRetry::new(
            || ok::<_, u8>(255).map_err(|_| RetryPolicy::Repeat::<String>),
            LinearRetryStrategy::default(),
        );
        assert_eq!(255, f.await.unwrap());
    }

    #[tokio::test]
    async fn test_error() {
        let f = FutureRetry::new(
            || err::<u8, _>(RetryPolicy::Fail("fail")),
            LinearRetryStrategy::default().attempts(1),
        );
        assert_eq!(f.await.unwrap_err().to_string(), "Fail: fail");
    }
}
