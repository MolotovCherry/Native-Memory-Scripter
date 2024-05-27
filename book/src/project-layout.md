# Project Layout

In the same directory that `native-memory-scriper.dll` is placed inside, the directory layout will be as follows
```
│   native-memory-scripter.dll
│   native-memory-scripter.log
│   native-memory-scripter.toml
│
└───native-scripts
    │   script1.py
    │
    ├───script2
    │       main.py
    │       module.py
    │
    └───_packages
        └───libs
                package1.py
```
At the top level is the dll's config file and a log file.

You may place a single use script named anything directly in the `native-scripts` folder, or in a separate folder with the name `main.py`. Each script is concurrently run _in a separate python interpreter_.

```admonish warning title="Module names"
Single module filenames or folder names of multi-module scripts can be named anything, but it's strongly suggested to stick to the same name as your mod.
```

#### Single module scripts

In the example tree above, `script1.py` is a single module script. It cannot import any local modules.

#### Multiple module scripts

If your mod requires multiple python files, you can choose to place your script in a folder instead. The initial file that gets run must be named `main.py`. Scripts in a directory may import any local module from their own directory, e.g. `import module`.

#### Libraries

The `_packages` directory is where modules are stored that can be used across multiple scripts. Every script can import from here with e.g. `from libs import package1`
