use std::any::TypeId;
use std::mem::{forget, transmute_copy};
use std::pin::Pin;
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

impl<'a, L, R, E, SL, SR> AsUpdateStream<'a, E> for Either<L, R>
where
    L: AsUpdateStream<'a, E, Stream = SL>,
    R: AsUpdateStream<'a, E, Stream = SR>,
    SL: 'a + Stream<Item = Result<Update, E>>,
    SR: 'a + Stream<Item = Result<Update, E>>,
    E: 'a,
{
    type Stream = Pin<Box<dyn Stream<Item = Result<Update, E>> + 'a>>;

    fn as_stream(&'a mut self) -> Self::Stream {
        match self {
            Either::Left(inner) => Box::pin(inner.as_stream()),
            Either::Right(inner) => Box::pin(inner.as_stream()),
        }
    }
}

impl<L, R, E, StL, StR> UpdateListener<E> for Either<L, R>
where
    L: UpdateListener<E, StopToken = StL>,
    R: UpdateListener<E, StopToken = StR>,
    StL: 'static + StopToken,
    StR: 'static + StopToken,
    Self: for<'a> AsUpdateStream<'a, E>,
{
    type StopToken = StL;

    fn stop_token(&mut self) -> Self::StopToken {
        assert_eq!(TypeId::of::<StL>(), TypeId::of::<StR>(), "type mismatch");
        match self {
            Either::Left(inner) => inner.stop_token(),
            Either::Right(inner) => unsafe {
                let t = inner.stop_token();
                let u = transmute_copy(&t);
                forget(t);
                u
            },
        }
    }

    fn hint_allowed_updates(&mut self, hint: &mut dyn Iterator<Item = AllowedUpdate>) {
        either!(self, ref mut inner => inner.hint_allowed_updates(hint));
    }

    fn timeout_hint(&self) -> Option<Duration> {
        either!(self, ref inner => inner.timeout_hint())
    }
}
