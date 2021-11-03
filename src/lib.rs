use std::env;

use teloxide::dispatching::update_listeners;
use teloxide::dispatching::update_listeners::UpdateListener;
use teloxide::prelude::*;
use teloxide::RequestError;

#[cfg(feature = "either")]
pub use crate::either::Either;

#[cfg(feature = "either")]
mod either;
#[cfg(feature = "webhook")]
mod webhook;

#[cfg(test)]
mod tests;

pub enum Listener {
    Polling,
    #[cfg(feature = "webhook")]
    Webhook(webhook::HTTPConfig),
}

impl Listener {
    /// # Panics
    /// Panics when webhook env vars are set but crate is compiled without webhook support.
    #[must_use]
    #[allow(unused_variables)]
    pub fn from_env() -> Self {
        if let (Ok(base), Ok(path), Ok(addr)) = (
            env::var("APP_WEBHOOK_URL"),
            env::var("APP_WEBHOOK_PATH"),
            env::var("APP_BIND_ADDR"),
        ) {
            #[cfg(not(feature = "webhook"))]
            panic!("webhook support not enabled");
            #[cfg(feature = "webhook")]
            Self::Webhook(webhook::HTTPConfig::new(
                base.as_str(),
                path.as_str(),
                addr.as_str(),
            ))
        } else {
            Self::Polling
        }
    }

    #[cfg(feature = "webhook")]
    #[allow(clippy::future_not_send)]
    pub async fn build<R>(
        self,
        bot: R,
    ) -> Either<impl UpdateListener<R::Err>, impl UpdateListener<R::Err>>
    where
        R: Requester<Err = RequestError> + 'static,
        <R as Requester>::GetUpdatesFaultTolerant: Send,
    {
        match self {
            Listener::Polling => Either::Left(update_listeners::polling_default(bot).await),
            Listener::Webhook(config) => Either::Right(webhook::listener(bot, config).await),
        }
    }

    #[cfg(not(feature = "webhook"))]
    #[allow(clippy::future_not_send)]
    pub async fn build<R>(self, bot: R) -> impl UpdateListener<R::Err>
    where
        R: Requester<Err = RequestError> + 'static,
        <R as Requester>::GetUpdatesFaultTolerant: Send,
    {
        match self {
            Listener::Polling => update_listeners::polling_default(bot).await,
        }
    }
}
