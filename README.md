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
[rpc_config]
url      = "http://127.0.0.1:9091/transmission/rpc"
username = "your-username"
password = "your-password"

[theme.general]
background = "#1a1b26"
foreground = "#c0caf5"

[theme.tabs]
active_fg   = "#7aa2f7"
active_bg   = "#24283b"
inactive_fg = "#565f89"
inactive_bg = "#1a1b26"
highlight   = "#f7768e"

[theme.table]
row_highlight_fg = "#24283b"
row_highlight_bg = "#7aa2f7"

[theme.progress_bar]
filled = "#7aa2f7"
empty  = "#24283b"
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
