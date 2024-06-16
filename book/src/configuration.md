# Configuration

The configuration uses [toml](https://toml.io/en/) and is stored at `native-memory-scripter.toml` alongside the dll.

## dev
This section contains developer settings.

| key      | type | description                         |
| -------- | ---- | ----------------------------------- |
| console  | bool | Shows or hides the debug console.   |
| dev_mode | bool | Shows or hides dev mode.[^dev_mode] |

[^dev_mode]: Dev mode is a special mode that spawns an interactive python interpreter window that you can script in while the game is running

## log
This section controls the built in logger.

| key     | type | description                               |
| ------- | ---- | ----------------------------------------- |
| level   | str  | Allows you to fine tune what gets logged. |
| targets | bool | Show or hide log targets.[^targets]       |

[^targets]: A target is a piece of information that describes what part of the code a particular message came from. The target is the code's namespace by default.
