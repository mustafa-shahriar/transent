use ratatui::style::Color;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    pub general: General,
    pub tabs: Tabs,
    pub table: TableTheme,
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
    pub highlight: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TableTheme {
    pub row_highlight_fg: String,
    pub row_highlight_bg: String,
}

impl Theme {
    pub fn color(s: &str) -> Color {
        // Accepts "#RRGGBB" or "0xRRGGBB"
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