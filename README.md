# transmission-tui

A terminal user interface (TUI) for managing torrents via Transmission, written in Rust using [ratatui](https://github.com/ratatui-org/ratatui).

## Features

- View and navigate torrents
- Switch tabs and focus areas
- Configurable keybindings
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
~/.config/transmission-tui/
```

### Keybindings (`key_config.toml`)

```toml
[keybindings]
"q" = "quit"
"Esc" = "quit"
"Ctrl+c" = "quit"
"Ctrl+j" = "focus_bottom"
"Ctrl+k" = "focus_top"
"Ctrl+l" = "focus_bottom"
"Ctrl+h" = "focus_top"
"h" = "tab_left"
"l" = "tab_right"
"j" = "row_down"
"k" = "row_up"
```

### Theme (`theme_config.toml`)

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
