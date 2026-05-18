use std::fs;

use ratatui::style::Color;
use serde::Deserialize;

use crate::util::get_conf_dir;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rpc_config: RpcConfig,
    pub theme: Theme,
}

#[derive(Debug, Deserialize, Clone)]
struct RawConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub theme: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RpcConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    pub general: General,
    pub tabs: Tabs,
    pub table: TableTheme,
    pub progress_bar: ProgressBar,
    pub details: Details,
}

#[derive(Debug, Deserialize, Clone)]
pub struct General {
    pub background: String,
    pub foreground: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tabs {
    pub active_fg: String,
    pub active_bg: String,
    pub inactive_fg: String,
    pub inactive_bg: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TableTheme {
    pub row_highlight_fg: String,
    pub row_highlight_bg: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProgressBar {
    pub filled: String,
    pub empty: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Details {
    pub card_bg: String,
    pub muted_fg: String,
    pub accent_fg: String,
    pub success_fg: String,
    pub border_color: String,
}

impl Theme {
    pub fn color(s: &str) -> Color {
        let s = s.trim_start_matches('#').trim_start_matches("0x");
        if let Ok(rgb) = u32::from_str_radix(s, 16) {
            Color::Rgb(
                ((rgb >> 16) & 0xFF) as u8,
                ((rgb >> 8) & 0xFF) as u8,
                (rgb & 0xFF) as u8,
            )
        } else {
            Color::White
        }
    }
}

fn resolve_theme(name: &str) -> Theme {
    let toml_str = match name {
        "catppuccin_mocha" => include_str!("../../themes/catppuccin_mocha.toml"),
        "dracula" => include_str!("../../themes/dracula.toml"),
        "gruvbox_dark" => include_str!("../../themes/gruvbox_dark.toml"),
        "nord" => include_str!("../../themes/nord.toml"),
        "rose_pine" => include_str!("../../themes/rose_pine.toml"),
        "github" => include_str!("../../themes/github.toml"),
        "github_dark" => include_str!("../../themes/github_dark.toml"),
        _ => include_str!("../../themes/tokyonight.toml"),
    };
    toml::from_str(toml_str).expect("Invalid theme file")
}

pub fn get_config() -> Config {
    let path = get_conf_dir().join("config.toml");
    let content = fs::read_to_string(&path).expect("config.toml not found");
    let raw: RawConfig = toml::from_str(&content).expect("Invalid config.toml");
    Config {
        rpc_config: RpcConfig {
            url: raw.url,
            username: raw.username,
            password: raw.password,
        },
        theme: resolve_theme(&raw.theme),
    }
}
