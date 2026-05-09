# transmission-tui

A terminal user interface (TUI) for managing torrents via Transmission, written in Rust using [ratatui](https://github.com/ratatui-org/ratatui).

## Features

- View and navigate torrents
- Keyboard-driven interface
- Tab-based navigation
- Configurable theme
- Transmission RPC support

## Installation

### Requirements

- Rust (latest stable recommended)
- `transmission-daemon` running with RPC enabled

### Build

```sh
git clone https://github.com/mustafa-shahriar/transent
cd transent
cargo build --release
```

### Run

```sh
./target/release/transent
```

## Configuration

Configuration files should be placed in:

```
~/.config/transent/config.toml
```

### Example Configuration

```toml
url      = "http://127.0.0.1:9091/transmission/rpc"
username = "your-username"
password = "your-password"

theme = "tokyonight"
# theme = "catppuccin_mocha"
# theme = "dracula"
# theme = "gruvbox_dark"
# theme = "nord"
# theme = "rose_pine"
```

## Notes

Make sure Transmission RPC is enabled in your `settings.json`:

```json
"rpc-enabled": true,
"rpc-username": "your-username",
"rpc-password": "your-password"
```

> **Note:** `rpc-username` and `rpc-password` must match your config.

Default RPC endpoint: `http://127.0.0.1:9091/transmission/rpc`
