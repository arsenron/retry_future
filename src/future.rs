use crate::error::RetryError;
use crate::retry_strategy::RetryStrategy;
use crate::RetryPolicy;
use futures::{ready, TryFuture};
use pin_project::pin_project;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::sleep;

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
    NeedsPolling(Poll<Output>),
}

#[pin_project]
pub struct FutureRetry<F, E, RS>
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

impl<F, E, RS> FutureRetry<F, E, RS>
where
    F: FutureFactory<E>,
    RS: RetryStrategy,
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
                    FutureState::WaitingForFuture { future: future_retry.factory.new() }
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
