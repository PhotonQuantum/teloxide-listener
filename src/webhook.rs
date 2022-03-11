//! Webhook update handler.

use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use teloxide::dispatching::stop_token::AsyncStopToken;
use teloxide::dispatching::update_listeners::{StatefulListener, UpdateListener};
use teloxide::requests::{Request, Requester};
use teloxide::types::Update;
use teloxide::RequestError;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;

/// HTTP config for receiving updates via webhook.
pub struct HTTPConfig {
    /// Base URL for the callback.
    pub base_url: Url,
    /// Path for the callback.
    pub path: String,
    /// Address to listen for updates.
    pub addr: SocketAddr,
}

impl HTTPConfig {
    fn full_url(&self) -> Url {
        self.base_url.join(self.path.as_str()).expect("invalid url")
    }
}

struct State<S, T> {
    stream: S,
    stop_tx: T,
}

impl<S, T> State<S, T> {
    fn stream_mut(&mut self) -> &mut S {
        &mut self.stream
    }
}

impl<S, T: Clone> State<S, T> {
    fn stop_tx(&mut self) -> T {
        self.stop_tx.clone()
    }
}

#[allow(clippy::future_not_send)]
#[doc(hidden)]
pub async fn listener<R>(bot: R, config: HTTPConfig) -> impl UpdateListener<R::Err>
where
    R: 'static + Requester<Err = RequestError>,
{
    tracing::info!("Starting webhook listener");

    bot.set_webhook(config.full_url())
        .send()
        .await
        .expect("unable to setup webhook");

    let (tx, rx) = unbounded_channel();

    let app = Router::new()
        .route(
            format!("/{}", config.path.trim_start_matches('/')).as_str(),
            post(move |Json(payload): Json<Update>| async move {
                tracing::debug!("Received update: {:?}", payload);
                tx.send(Ok(payload))
                    .expect("Cannot send an incoming update from the webhook");

                StatusCode::OK
            }),
        )
        .route(
            "/health-check",
            get(|| async {
                tracing::debug!("Received health check request");
                StatusCode::OK
            }),
        );

    let (stop_tx, stop_rx) = AsyncStopToken::new_pair();

    let srv = axum::Server::bind(&config.addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(stop_rx);

    tokio::spawn(srv);

    tracing::info!(%config.base_url, %config.path, %config.addr, "Webhook listening for updates");

    let stream = UnboundedReceiverStream::new(rx);

    StatefulListener::new(State { stream, stop_tx }, State::stream_mut, State::stop_tx)
}
