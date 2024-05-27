# Project Layout and Setup

In the same directory `native-memory-scripter.dll` is inside, the directory layout will be as follows
```
│   native-memory-scripter.dll
│   native-memory-scripter.log
│   native-memory-scripter.toml
│
└───native-scripts
    │
    ├───script
    │       plugin.toml
    │       main.py
    │       module.py
    │
    └───_packages
        └───libs
                package1.py
```
At the top level is the dll's config file and a log file.

```admonish warning title="Plugin folder names"
Plugin folder names can be named anything, but it's strongly recommended to stick to the same name as your plugin.
```

#### Native Plugin Scripts

Each native plugin script must be placed in a folder, along with a `plugin.toml` describing the plugin and a `main.py` which is the plugin's entry point. Each plugin is concurrently run _in a separate python interpreter_. Scripts in a directory may import any local module from their own directory, e.g. `import module`.

#### Plugin details
Every plugin must provide a `plugin.toml` detailing the plugin details
```toml
[plugin]
name = "My Plugin Name"
author = "My Plugin Author"
description = "Description of my plugin"
version = "0.1.0"
```

#### Libraries

The `_packages` directory is where modules are stored that can be used across multiple scripts. Every script can import from here with e.g. `from libs import package1`
