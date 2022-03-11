//! A listener extension for [teloxide](https://github.com/teloxide/teloxide).
//!
//! Currently supports the following modes:
//! - `polling`
//! - `webhook` (axum, need to be enabled by feature flag)
//!
//! ## Usage
//!
//! Construct a `Listener` builder, build it, and pass it to `with_listener` versions of teloxide functions (e.g., [`repl_with_listener`](teloxide::dispatching2::repls::repl_with_listener)).
//!
//! There are two ways to construct a `Listener` builder.
//!
//! ### From environment variables
//!
//! [`Listener::from_env`](Listener::from_env) can be used to construct a `Listener` from environment variables.
//!
//! If compiled with `webhook` feature enabled, it tries to read `TELOXIDE_WEBHOOK_URL`, `TELOXIDE_WEBHOOK_PATH`, and `TELOXIDE_BIND_ADDR` to build a webhook updates listener first.
//!
//! Otherwise, it falls back to long polling updates listener.
//!
//! To customize the `TELOXIDE_` prefix, use [`Listener::from_env_with_prefix`](Listener::from_env_with_prefix).
//!
//! ### Constructing a `Listener` manually
//!
//! - [`Listener::Polling`](Listener::Polling) - a long polling updates listener.
//! - [`Listener::Webhook`](Listener::Webhook) - a webhook updates listener.
//!
//! ## Example
//!
//! ```rust
//! # use teloxide::{Bot, respond};
//! # use teloxide::requests::{Request, Requester};
//! # use teloxide::types::Message;
//! use teloxide_listener::Listener;
//!
//! # #[test]
//! # fn test() {
//!     # let bot = Bot::new("");
//!     #
//!     # drop(async move {
//! let listener = Listener::from_env().build(bot.clone());
//!
//! teloxide::repls2::repl_with_listener(
//!     bot,
//!     |msg: Message, bot: Bot| async move {
//!         bot.send_message(msg.chat.id, "pong").send().await?;
//!         respond(())
//!     },
//!     listener,
//! )
//! #    })
//! # }
//! ```
//!
//! ## License
//!
//! This project is licensed under the MIT license.
#![deny(missing_docs)]

use std::env;

use teloxide::dispatching::update_listeners;
use teloxide::dispatching::update_listeners::UpdateListener;
use teloxide::requests::Requester;
use teloxide::RequestError;

#[cfg(feature = "either")]
#[doc(hidden)]
pub use crate::either::Either;

#[cfg(feature = "either")]
#[doc(hidden)]
mod either;
#[cfg(feature = "webhook")]
pub mod webhook;

#[cfg(test)]
mod tests;

/// Builder for `UpdateListener` instance.
pub enum Listener {
    /// Polling listener.
    Polling,
    /// Webhook listener.
    #[cfg(feature = "webhook")]
    Webhook(webhook::HTTPConfig),
}

impl Listener {
    /// Creates a new `Listener` from environment variables with `TELOXIDE_` prefix.
    ///
    /// To create a Webhook listener, you need to set `TELOXIDE_WEBHOOK_URL`, `TELOXIDE_WEBHOOK_PATH` and `TELOXIDE_BIND_ADDR` environment variables.
    /// If one of them is not set, it will fallback to polling.
    ///
    /// If this crate is compiled without `webhook` feature, it will fallback to polling.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_with_prefix("TELOXIDE_")
    }

    /// Creates a new `Listener` from environment variables with given prefix.
    ///
    /// To create a Webhook listener, you need to set `{PREFIX}WEBHOOK_URL`, `{PREFIX}WEBHOOK_PATH` and `{PREFIX}BIND_ADDR` environment variables.
    /// If one of them is not set, it will fallback to polling.
    ///
    /// If this crate is compiled without `webhook` feature, it will fallback to polling.
    #[must_use]
    #[allow(unused_variables)]
    pub fn from_env_with_prefix(prefix: &str) -> Self {
        if let (Ok(base), Ok(path), Ok(addr)) = (
            env::var(format!("{}WEBHOOK_URL", prefix)),
            env::var(format!("{}WEBHOOK_PATH", prefix)),
            env::var(format!("{}BIND_ADDR", prefix)),
        ) {
            #[cfg(not(feature = "webhook"))]
            {
                tracing::error!("webhook support not enabled, fallback to polling");
                Self::Polling
            }
            #[cfg(feature = "webhook")]
            Self::Webhook(webhook::HTTPConfig {
                base_url: base.parse().expect("invalid base url"),
                path,
                addr: addr.parse().expect("invalid bind address"),
            })
        } else {
            Self::Polling
        }
    }

    /// Build a `UpdateListener` implementation from this `Listener`.
    ///
    /// See crate documentation for more information.
    #[cfg(feature = "webhook")]
    #[allow(clippy::future_not_send)]
    pub async fn build<R>(
        self,
        bot: R,
    ) -> Either<impl UpdateListener<R::Err>, impl UpdateListener<R::Err>>
    where
        R: Requester<Err = RequestError> + Send + 'static,
        <R as Requester>::GetUpdates: Send,
    {
        match self {
            Listener::Polling => Either::Left(update_listeners::polling_default(bot).await),
            Listener::Webhook(config) => Either::Right(webhook::listener(bot, config).await),
        }
    }

    /// Build a `UpdateListener` implementation from this `Listener`.
    ///
    /// See crate documentation for more information.
    #[cfg(not(feature = "webhook"))]
    #[allow(clippy::future_not_send)]
    pub async fn build<R>(self, bot: R) -> impl UpdateListener<R::Err>
    where
        R: Requester<Err = RequestError> + Send + 'static,
        <R as Requester>::GetUpdates: Send,
    {
        match self {
            Listener::Polling => update_listeners::polling_default(bot).await,
        }
    }
}
