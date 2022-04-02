use futures_core::Stream;
use std::pin::Pin;
use std::time::Duration;
use teloxide::dispatching::stop_token::StopToken;
use teloxide::dispatching::update_listeners::{AsUpdateStream, UpdateListener};
use teloxide::types::{AllowedUpdate, Update};

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

macro_rules! either {
    ($value:expr, $pattern:pat => $result:expr) => {
        match $value {
            Either::Left($pattern) => $result,
            Either::Right($pattern) => $result,
        }
    };
}

impl<'a, L, R, E, SL, SR> AsUpdateStream<'a, E> for Either<L, R>
where
    L: AsUpdateStream<'a, E, Stream = SL>,
    R: AsUpdateStream<'a, E, Stream = SR>,
    SL: 'a + Stream<Item = Result<Update, E>> + Send,
    SR: 'a + Stream<Item = Result<Update, E>> + Send,
    E: 'a,
{
    type Stream = Pin<Box<dyn Stream<Item = Result<Update, E>> + Send + 'a>>;

    fn as_stream(&'a mut self) -> Self::Stream {
        either!(self, ref mut inner => Box::pin(inner.as_stream()))
    }
}

impl<L, R, E> UpdateListener<E> for Either<L, R>
where
    L: UpdateListener<E>,
    R: UpdateListener<E>,
    Self: for<'a> AsUpdateStream<'a, E>,
{
    type StopToken = Either<L::StopToken, R::StopToken>;

    fn stop_token(&mut self) -> Self::StopToken {
        match self {
            Either::Left(inner) => Either::Left(inner.stop_token()),
            Either::Right(inner) => Either::Right(inner.stop_token()),
        }
    }

    fn hint_allowed_updates(&mut self, hint: &mut dyn Iterator<Item = AllowedUpdate>) {
        either!(self, ref mut inner => inner.hint_allowed_updates(hint));
    }

    fn timeout_hint(&self) -> Option<Duration> {
        either!(self, ref inner => inner.timeout_hint())
    }
}

impl<L: StopToken, R: StopToken> StopToken for Either<L, R> {
    fn stop(self) {
        either!(self, inner => inner.stop());
    }
}
