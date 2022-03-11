#[test]
#[cfg(feature = "webhook")]
fn must_update_listener() {
    use teloxide::dispatching2::{Dispatcher, UpdateFilterExt};
    use teloxide::dptree::endpoint;
    use teloxide::error_handlers::LoggingErrorHandler;
    use teloxide::respond;
    use teloxide::types::{Message, Update};
    use teloxide::Bot;

    use crate::webhook::HTTPConfig;
    use crate::Listener;

    let listener = Listener::Webhook(HTTPConfig {
        base_url: "http://example.com".parse().unwrap(),
        path: String::new(),
        addr: "0.0.0.0:8080".parse().unwrap(),
    });

    let bot = Bot::new("");
    let mut dispatcher = Dispatcher::builder(
        bot.clone(),
        Update::filter_message().branch(endpoint(
            |_msg: Message, _bot: Bot| async move { respond(()) },
        )),
    )
    .build();
    let err_handler = LoggingErrorHandler::with_custom_text("An error from the update listener");

    drop(async {
        dispatcher
            .dispatch_with_listener(listener.build(bot).await, err_handler)
            .await;
    });
}
