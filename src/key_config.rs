use core::panic;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::str::FromStr;

use crate::components::util;

#[derive(Debug, Clone)]
pub enum Actions {
    Quit,
    FocusTop,
    FocusBottom,
    TabLeft,
    TabRight,
    RowDown,
    RowUp,
    Resume,
    Pause,
    Delete,
    DeleteWithData,
    AddTorrent,
}

impl FromStr for Actions {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quit" => Ok(Actions::Quit),
            "focus_bottom" => Ok(Actions::FocusBottom),
            "focus_top" => Ok(Actions::FocusTop),
            "tab_left" => Ok(Actions::TabLeft),
            "tab_right" => Ok(Actions::TabRight),
            "row_down" => Ok(Actions::RowDown),
            "row_up" => Ok(Actions::RowUp),
            "pause" => Ok(Actions::Pause),
            "resume" => Ok(Actions::Resume),
            "delete" => Ok(Actions::Delete),
            "delete_with_data" => Ok(Actions::DeleteWithData),
            "add_torrent" => Ok(Actions::AddTorrent),
            _ => Err(()),
        }
    }
}

pub fn load_keymap() -> HashMap<String, Actions> {
    let path = util::get_conf_dir().join("key_config.toml");
    let path = path.to_str().unwrap();
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("❌ Could not read key config file: {}", path));

    let raw: toml::Value = toml::from_str(&content)
        .unwrap_or_else(|_| panic!("❌ Invalid TOML format in key config file: {}", path));

    let keybindings = raw
        .get("keybindings")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("❌ Missing [keybindings] section in config: {}", path));

    let mut map = HashMap::new();

    for (key, value) in keybindings {
        let key_norm = key.to_ascii_lowercase();

        if !is_valid_key(&key_norm) {
            panic!(
                "❌ Invalid key format `{}` in file `{}`. Expected key or modifier+key (e.g. 'q' or 'ctrl+j')",
                key, path
            );
        }

        let action_str = value.as_str().unwrap_or_else(|| {
            panic!(
                "❌ Invalid value type for key `{}` in `{}`. Expected a string action (e.g. 'quit').",
                key, path
            )
        });

        let action = action_str.parse::<Actions>().unwrap_or_else(|_| {
            panic!(
                "❌ Invalid action `{}` for key `{}` in file `{}`. Allowed actions: quit, focus_top, focus_bottom, tab_left, tab_right, row_down, row_up.",
                action_str, key, path
            )
        });

        map.insert(key_norm, action);
    }

    map
}

/// Convert a `KeyEvent` into a string format like "ctrl+j"
pub fn keyevent_to_string(key: &KeyEvent) -> String {
    let mut s = String::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        s.push_str("ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        s.push_str("alt+");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        s.push_str("shift+");
    }
    match key.code {
        KeyCode::Char(c) => s.push(c.to_ascii_lowercase()),
        KeyCode::Esc => s.push_str("esc"),
        KeyCode::Enter => s.push_str("enter"),
        KeyCode::Tab => s.push_str("tab"),
        _ => {}
    }
    s
}

fn is_valid_key(key: &str) -> bool {
    let key = key.trim().to_ascii_lowercase();
    let parts: Vec<&str> = key.split('+').collect();

    if parts.len() == 1 {
        if key.is_ascii() {
            return true;
        }
        return false;
    }

    if parts.len() != 2 {
        return false;
    }

    if (parts[0] == "ctrl" || parts[0] == "alt" || parts[0] == "shift") && parts[1].is_ascii() {
        return true;
    }

    return false;
}
