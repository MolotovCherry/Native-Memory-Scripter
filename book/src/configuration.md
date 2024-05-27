# Configuration

The configuration uses [toml](https://toml.io/en/) and is stored at `native-memory-scripter.toml` alongside the dll.

## dev
This section contains developer settings

#### console: bool
Shows or hides the debug console

#### dev_mode: bool
Shows or hides dev mode. Dev mode is a special mode that spawns an interactive python interpreter window that you can script in while the game is running

## log
This section controls the built in logger

#### level: str
Allows you to fine tune what gets logged

This is powered by [tracing](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax) under the hood. You can see some examples there. For example, if you wanted to only show log output for your script, you could use a level of `[script{name="my-script"}]=info`

#### targets: bool
Show or hide log targets. A target is a piece of information that describes what part of the code a particular message came from. The target is the code's namespace by default.
