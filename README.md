# teloxide-listener

A listener extension for [teloxide](https://github.com/teloxide/teloxide).

Currently supports the following modes:
- `polling`
- `webhook` (need to be enabled by feature flag)

## Usage

Enable the feature flag `webhook` in your `Cargo.toml`, and ensure that `TELOXIDE_WEBHOOK_URL`, `TELOXIDE_WEBHOOK_PATH`, and `TELOXIDE_BIND_ADDR` environment variables are set.

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