use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{ready, TryFuture};
use pin_project::pin_project;
use tokio::time::sleep;

use crate::error::RetryError;
use crate::retry_strategy::RetryStrategy;
use crate::RetryPolicy;

/// Spawns a new `Future` by `new` method in case of
/// error returned from a `Future`
pub trait FutureFactory<E> {
    type Future: TryFuture<Error = RetryPolicy<E>>;

    #[allow(clippy::wrong_self_convention)]
    fn new(&mut self) -> Self::Future;
}

impl<T, Fut, E> FutureFactory<E> for T
where
    T: Unpin + FnMut() -> Fut,
    Fut: TryFuture<Error = RetryPolicy<E>>,
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
    /// When this enum variant is matched, we immediately return
    /// from `poll`
    NeedsPolling(Poll<Output>),
}

/// A future which is trying to resolve inner future
/// until it exits successfully or return an [error](crate::error::RetryError).
///
/// The main point is that you handle all the logic **inside** your future
/// and construct a helper type or use one of existing which implements
/// [RetryStrategy](crate::retry_strategy::RetryStrategy) trait
/// which is responsible for configuring retry mechanism
#[pin_project]
pub struct AsyncRetry<F, E, RS>
where
    F: FutureFactory<E>,
    RS: RetryStrategy,
{
    factory: F,
    retry_strategy: RS,
    attempt: usize,
    #[pin]
    state: FutureState<F::Future, <Self as Future>::Output>,
    phantom: PhantomData<E>,
}

impl<F, E, RS> AsyncRetry<F, E, RS>
where
    F: FutureFactory<E>,
    RS: RetryStrategy,
{
    /// [FutureFactory](FutureFactory) has a blanket implementation
    /// for FnMut closures. This means that you can pass a closure instead
    /// of implementing [FutureFactory](FutureFactory) for your type.
    ///
    /// See examples to understand how to use this.
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

impl<F, E, RS> Future for AsyncRetry<F, E, RS>
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
                                Ok(duration) => FutureState::TimerActive { delay: sleep(duration) },
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
                                move_to_next_state_depending_on_retry_strategy(Some(
                                    any.context(format!("Failed after repeating {attempt} times",)),
                                ))
                            }
                        };
                        *future_retry.attempt += 1;
                        new_state
                    }
                },
                FutureStateProj::TimerActive { delay } => {
                    ready!(delay.poll(cx));
                    FutureState::WaitingForFuture { future: future_retry.factory.new() }
                }
                FutureStateProj::NeedsPolling(poll) => {
                    // move from &mut T to original T. It is ok as we immediately return
                    let output = std::mem::replace(poll, Poll::Pending);
                    return output;
                }
            };

            self.as_mut().project().state.set(new_state);
        }
    }
}
