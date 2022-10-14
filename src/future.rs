use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{ready, TryFuture};
use pin_project::pin_project;
use tokio::time::sleep;

use crate::error::RetryError;
use crate::retry_strategy::RetryStrategy;
use crate::RetryPolicy;

pub trait FutureFactory<E> {
    type Future: TryFuture<Error = RetryPolicy<E>>;

    fn spawn(&mut self) -> Self::Future;
}

impl<T, Fut, E> FutureFactory<E> for T
where
    T: Unpin + FnMut() -> Fut,
    Fut: TryFuture<Error = RetryPolicy<E>>,
{
    type Future = Fut;

    fn spawn(&mut self) -> Fut {
        self()
    }
}

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
pub struct AsyncRetry<F, E, RS>
where
    F: FutureFactory<E>
{
    factory: F,
    retry_strategy: RS,
    attempts_before: usize,
    #[pin]
    state: FutureState<F::Future>,
    errors: Vec<RetryPolicy<E>>,
}

impl<F, E, RS> AsyncRetry<F, E, RS>
where
    F: FutureFactory<E>,
{
    /// [FutureFactory](FutureFactory) has a blanket implementation
    /// for FnMut closures. This means that you can pass a closure instead
    /// of implementing [FutureFactory](FutureFactory) for your type.
    ///
    /// See examples to understand how to use this.
    pub fn new(mut factory: F, retry_strategy: RS) -> Self {
        let future = factory.spawn();
        Self {
            factory,
            retry_strategy,
            state: FutureState::WaitingForFuture { future },
            attempts_before: 0,
            errors: Vec::new(),
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
            let async_retry = self.as_mut().project();
            let new_state = match async_retry.state.project() {
                FutureStateProj::WaitingForFuture { future } => match ready!(future.try_poll(cx)) {
                    Ok(t) => {
                        *async_retry.attempts_before = 0;
                        return Poll::Ready(Ok(t));
                    }
                    Err(err) => {
                        async_retry.errors.push(err);
                        let err = async_retry.errors.last().unwrap(); // cannot panic as we just pushed to vec
                        let new_state = match err {
                            RetryPolicy::Repeat(_) => {
                                let check_attempt_result = async_retry
                                    .retry_strategy
                                    .check_attempt(*async_retry.attempts_before);
                                match check_attempt_result {
                                    Ok(duration) => {
                                        FutureState::TimerActive { delay: sleep(duration) }
                                    }
                                    Err(_) => {
                                        let errors =
                                            std::mem::take(async_retry.errors);
                                        return Poll::Ready(Err(RetryError { errors }));
                                    }
                                }
                            }
                            RetryPolicy::Fail(_) => {
                                let errors = std::mem::take(async_retry.errors);
                                return Poll::Ready(Err(RetryError { errors }));
                            }
                        };
                        *async_retry.attempts_before += 1;
                        new_state
                    }
                },
                FutureStateProj::TimerActive { delay } => {
                    ready!(delay.poll(cx));
                    FutureState::WaitingForFuture { future: async_retry.factory.spawn() }
                }
            };

            self.as_mut().project().state.set(new_state);
        }
    }
}
