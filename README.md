# transmission-tui

A terminal user interface (TUI) for managing torrents via Transmission, written in Rust using [ratatui](https://github.com/ratatui-org/ratatui).

## Features

- View and navigate torrents
- Switch tabs and focus areas
- Configurable theme

## Installation

### Requirements

- Rust
- `transmission-daemon` running with RPC enabled

### Build

```sh
git clone https://github.com/mustafa-shahriar/transent
cd transent
cargo build --release
````

### Run

```sh
./target/release/transent
```

## Configuration

Place configuration files in:


```
~/.config/transent/
```
### Theme (`config.toml`)

```toml
rpc_url = "transmission-daemon url with password"

```

```
~/.config/transent/
```
### Theme (`theme.toml`)

```toml
[general]
background = "#1a1b26"
foreground = "#c0caf5"

[tabs]
active_fg = "#7aa2f7"
active_bg = "#24283b"
inactive_fg = "#565f89"
inactive_bg = "#1a1b26"
highlight = "#f7768e"

[table]
row_highlight_fg = "#24283b"
row_highlight_bg = "#7aa2f7"
```

## License

MIT
