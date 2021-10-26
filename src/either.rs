use std::time::Duration;

use futures_core::Stream;
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

impl<'a, L, R, E, S> AsUpdateStream<'a, E> for Either<L, R>
where
    L: AsUpdateStream<'a, E, Stream = S>,
    R: AsUpdateStream<'a, E, Stream = S>,
    S: Stream<Item = Result<Update, E>> + 'a,
{
    type Stream = S;

    fn as_stream(&'a mut self) -> Self::Stream {
        either!(self, ref mut inner => inner.as_stream())
    }
}

impl<L, R, E, St> UpdateListener<E> for Either<L, R>
where
    L: UpdateListener<E, StopToken = St>,
    R: UpdateListener<E, StopToken = St>,
    St: StopToken,
    Self: for<'a> AsUpdateStream<'a, E>,
{
    type StopToken = St;

    fn stop_token(&mut self) -> Self::StopToken {
        either!(self, ref mut inner => inner.stop_token())
    }

    fn hint_allowed_updates(&mut self, hint: &mut dyn Iterator<Item = AllowedUpdate>) {
        either!(self, ref mut inner => inner.hint_allowed_updates(hint));
    }

    fn timeout_hint(&self) -> Option<Duration> {
        either!(self, ref inner => inner.timeout_hint())
    }
}
