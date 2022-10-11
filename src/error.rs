use std::fmt::{Debug, Display, Formatter};

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
