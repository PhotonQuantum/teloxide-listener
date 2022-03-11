use std::net::SocketAddr;
use std::str::FromStr;

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use teloxide::dispatching::stop_token::AsyncStopToken;
use teloxide::dispatching::update_listeners::{StatefulListener, UpdateListener};
use teloxide::prelude::*;
use teloxide::types::Update;
use teloxide::RequestError;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;

pub struct HTTPConfig {
    pub base_url: Url,
    pub path: String,
    pub addr: SocketAddr,
}

impl HTTPConfig {
    #[must_use]
    pub fn new(base_url: &str, path: &str, addr: &str) -> Self {
        Self {
            base_url: Url::parse(base_url).expect("invalid base url"),
            path: path.to_string(),
            addr: SocketAddr::from_str(addr).expect("invalid bind addr"),
        }
    }
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
pub async fn listener<R>(bot: R, config: HTTPConfig) -> impl UpdateListener<R::Err>
where
    R: 'static + Requester<Err = RequestError>,
{
    bot.set_webhook(config.full_url())
        .send()
        .await
        .expect("unable to setup webhook");

    let (tx, rx) = unbounded_channel();

    let app = Router::new()
        .route(
            format!("/{}", config.path.trim_start_matches('/')).as_str(),
            post(move |Json(payload): Json<Update>| async move {
                tx.send(Ok(payload))
                    .expect("Cannot send an incoming update from the webhook");

                StatusCode::OK
            }),
        )
        .route("/health-check", get(|| async { StatusCode::OK }));

    let (stop_tx, stop_rx) = AsyncStopToken::new_pair();

    let srv = axum::Server::bind(&config.addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(stop_rx);

    tokio::spawn(srv);

    let stream = UnboundedReceiverStream::new(rx);

    StatefulListener::new(State { stream, stop_tx }, State::stream_mut, State::stop_tx)
}
