# teloxide-listener

A listener extension for [teloxide](https://github.com/teloxide/teloxide).

Currently supports the following modes:
- `polling`
- `webhook` (axum, need to be enabled by feature flag)

## Usage

Construct a `Listener` builder, build it, and pass it to `with_listener` versions of teloxide functions (e.g., [`repl_with_listener`](teloxide::dispatching2::repls::repl_with_listener)).

There are two ways to construct a `Listener` builder.

### From environment variables

[`Listener::from_env`](Listener::from_env) can be used to construct a `Listener` from environment variables.

If compiled with `webhook` feature enabled, it tries to read `TELOXIDE_WEBHOOK_URL`, `TELOXIDE_WEBHOOK_PATH`, and `TELOXIDE_BIND_ADDR` to build a webhook updates listener first.

Otherwise, it falls back to long polling updates listener.

To customize the `TELOXIDE_` prefix, use [`Listener::from_env_with_prefix`](Listener::from_env_with_prefix).

### Constructing a `Listener` manually

- [`Listener::Polling`](Listener::Polling) - a long polling updates listener.
- [`Listener::Webhook`](Listener::Webhook) - a webhook updates listener.

## Example

```rust
use teloxide_listener::Listener;

let listener = Listener::from_env().build(bot.clone());

teloxide::repls2::repl_with_listener(
    bot,
    |msg: Message, bot: Bot| async move {
        bot.send_message(msg.chat.id, "pong").send().await?;
        respond(())
    },
    listener,
)
```

## License

This project is licensed under the [MIT license](LICENSE).