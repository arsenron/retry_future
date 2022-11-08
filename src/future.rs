use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{ready, TryFuture};
use pin_project::pin_project;
use tokio::time::sleep;

use crate::error::RetryError;
use crate::retry_strategy::RetryStrategy;
use crate::RetryPolicy;

#[pin_project(project = FutureStateProj)]
enum FutureState<Fut> {
    WaitingForFuture {
        #[pin]
        future: Fut,
    },
    TimerActive {
        #[pin]
        delay: tokio::time::Sleep,
    },
}

/// A future which is trying to resolve inner future
/// until it exits successfully or return an [error](crate::error::RetryError).
///
/// The main point is that you handle all the logic **inside** your future
/// and construct a helper type or use one of existing which implements
/// [RetryStrategy](crate::retry_strategy::RetryStrategy) trait
/// which is responsible for configuring retry mechanism
#[pin_project]
pub struct RetryFuture<F, Fut, E, RS> {
    factory: F,
    retry_strategy: RS,
    attempts_before: usize,
    #[pin]
    state: FutureState<Fut>,
    errors: Vec<RetryPolicy<E>>,
}

impl<F, Fut, E, RS> RetryFuture<F, Fut, E, RS>
where
    F: Unpin + FnMut() -> Fut,
{
    pub fn new(mut factory: F, retry_strategy: RS) -> Self {
        let future = factory();
        Self {
            factory,
            retry_strategy,
            state: FutureState::WaitingForFuture { future },
            attempts_before: 0,
            errors: Vec::new(),
        }
    }
}

impl<F, Fut, E, RS> Future for RetryFuture<F, Fut, E, RS>
where
    F: Unpin + FnMut() -> Fut,
    Fut: TryFuture<Error = RetryPolicy<E>>,
    E: Debug,
    RS: RetryStrategy,
{
    type Output = Result<Fut::Ok, RetryError<E>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            let retry_future = self.as_mut().project();
            let new_state = match retry_future.state.project() {
                FutureStateProj::WaitingForFuture { future } => match ready!(future.try_poll(cx)) {
                    Ok(t) => {
                        *retry_future.attempts_before = 0;
                        return Poll::Ready(Ok(t));
                    }
                    Err(err) => {
                        #[cfg(feature = "log")]
                        log::trace!("Error returned from future - {err:?}");
                        retry_future.errors.push(err);
                        let err = retry_future.errors.last().unwrap(); // cannot panic as we just pushed to vec
                        let new_state = match err {
                            RetryPolicy::Retry(maybe_err) => {
                                if matches!(maybe_err, Some(e) if e.is_early_returned)
                                    && !retry_future.retry_strategy.retry_early_returned_errors()
                                {
                                    return Poll::Ready(Err(RetryError {
                                        errors: std::mem::take(retry_future.errors),
                                    }));
                                }
                                let check_attempt_result = retry_future
                                    .retry_strategy
                                    .check_attempt(*retry_future.attempts_before);
                                match check_attempt_result {
                                    Ok(duration) => {
                                        FutureState::TimerActive { delay: sleep(duration) }
                                    }
                                    Err(_) => {
                                        return Poll::Ready(Err(RetryError {
                                            errors: std::mem::take(retry_future.errors),
                                        }));
                                    }
                                }
                            }
                            RetryPolicy::Fail(_) => {
                                return Poll::Ready(Err(RetryError {
                                    errors: std::mem::take(retry_future.errors),
                                }));
                            }
                        };
                        *retry_future.attempts_before += 1;
                        new_state
                    }
                },
                FutureStateProj::TimerActive { delay } => {
                    ready!(delay.poll(cx));
                    FutureState::WaitingForFuture { future: (retry_future.factory)() }
                }
            };

            self.as_mut().project().state.set(new_state);
        }
    }
}
