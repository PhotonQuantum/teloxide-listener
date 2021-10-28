use std::marker::PhantomData;
use std::pin::Pin;
use std::time::Duration;

use futures_core::Stream;
use teloxide::dispatching::stop_token::StopToken;
use teloxide::dispatching::update_listeners::{AsUpdateStream, UpdateListener};
use teloxide::types::{AllowedUpdate, Update};

pub enum Either<'s, L, R> {
    Left((L, PhantomData<&'s L>)),
    Right((R, PhantomData<&'s R>)),
}

impl<'s, L, R> Either<'s, L, R> {
    pub fn new_left(l: L) -> Self {
        Either::Left((l, PhantomData))
    }
    pub fn new_right(r: R) -> Self {
        Either::Right((r, PhantomData))
    }
}

macro_rules! either {
    ($value:expr, $pattern:pat => $result:expr) => {
        match $value {
            Either::Left(($pattern, _)) => $result,
            Either::Right(($pattern, _)) => $result,
        }
    };
}

impl<'a, 's, L, R, E, SL, SR> AsUpdateStream<'a, E> for Either< 's, L, R>
where
    L: AsUpdateStream<'a, E, Stream = SL>,
    R: AsUpdateStream<'a, E, Stream = SR>,
    SL: 'a + Stream<Item = Result<Update, E>>,
    SR: 'a + Stream<Item = Result<Update, E>>,
    E: 'a,
{
    type Stream = Pin<Box<dyn Stream<Item = Result<Update, E>> + 'a>>;

    fn as_stream(&'a mut self) -> Self::Stream {
        either!(self, ref mut inner => Box::pin(inner.as_stream()))
    }
}

impl<'s, L, R, E, StL, StR> UpdateListener<E> for Either<'s, L, R>
where
    L: UpdateListener<E, StopToken = StL>,
    R: UpdateListener<E, StopToken = StR>,
    StL: 's + StopToken,
    StR: 's + StopToken,    
    Self: for<'a> AsUpdateStream<'a, E>,
{
    type StopToken = Box<dyn StopToken + 's>;

    fn stop_token(&mut self) -> Self::StopToken {
        either!(self, ref mut inner => Box::new(inner.stop_token()))
    }

    fn hint_allowed_updates(&mut self, hint: &mut dyn Iterator<Item = AllowedUpdate>) {
        either!(self, ref mut inner => inner.hint_allowed_updates(hint));
    }

    fn timeout_hint(&self) -> Option<Duration> {
        either!(self, ref inner => inner.timeout_hint())
    }
}
