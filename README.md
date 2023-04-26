# fw

A very simple tool to watch files and execute commands on transitions.

Currently, there are no pre-compiled binaries, but you can install the tool using cargo.
```
cargo install --git https://github.com/zekrotja/fw fw
```

## Why?

This tool is mostly purpose built to simplify and speed up some task during my development at work by restarting services when re-building modules. But still the target of the project is to make it as general purpose as possible, so that other automation tasks can be realized using this tool. Feel free to contribute ideas and bugs via [the issues](https://github.com/zekroTJA/fw/issues) or by creating [a pull request](https://github.com/zekroTJA/fw/compare).

## Configuration

Configuration can be provided as `TOML`, `YML` or `JSON` file either in the current working directory as `fw.*` or in the users home config directory, which maps to the following directories depending on the OS.
- Linux: `$HOME/.config/fw/config.*`
- macOS: `$HOME/Library/Application Support/de.zekro.fw/config.*`
- Windows: `%APPDATA%/zekro/fw/config.*`

Following, you can see an example configuration in `YML` format.
```yml
# The interval in which files will be checked
# for changes in milliseconds.
# optional
# default: 1000
check_interval_ms: 1000

# Conbinations of targets with commands
# which are executed for each target if
# it matches.
# required
actions:
    # List of files to be watched and
    # transitions which will trigger the
    # command execution.
    # required
  - targets:
      - "some/path/file"
        # The file path of the watched file.
        # required
      - path: "another/path/file"
        # The list of transitions on the file
        # triggering the command execution.
        # optional
        # default: all transitions
        transitions:
          - "created"
          - "modified"
          - "deleted"
      - path: "/last/path/file"
    commands:
      - 'sh -c "echo hello world!"' 
        # The command to be executed.
        # required
      - cmd: "cargo build"
        # The directory in which the command
        # will be executed.
        # optional
        # default: "./"
        cwd: "/dir/to/app"
        # Defaultly, the next command in line
        # will only be executed after the previous
        # one has resulted. When this is set to
        # true though, the next command is
        # executed immediately after calling 
        # the command and the current command
        # is executed in the background.
        # optional
        # default: false
        async: true
```