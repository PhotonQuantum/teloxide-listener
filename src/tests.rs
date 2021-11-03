use std::mem::MaybeUninit;

use teloxide::prelude::*;

use crate::Listener;

#[test]
#[cfg(feature = "webhook")]
fn must_update_listener() {
    let listener: MaybeUninit<Listener> = MaybeUninit::uninit();
    let listener = unsafe { listener.assume_init() };

    let bot = Bot::new("");
    let mut dispatcher = teloxide::dispatching::Dispatcher::new(bot.clone());
    let err_handler = LoggingErrorHandler::with_custom_text("An error from the update listener");

    drop(async {
        dispatcher
            .dispatch_with_listener(listener.build(bot).await, err_handler)
            .await
    });
}
