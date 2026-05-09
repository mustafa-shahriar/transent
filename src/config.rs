use ratatui::style::Color;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rpc_config: RpcConfig,
    pub theme: Theme,
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
