use crate::components::util::{expand_path, get_entries};
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Clear, Padding, Table, TableState};
use ratatui::{Frame, layout::Rect, widgets::Row};
use std::fs::DirEntry;

pub struct FilePicker {
    pub path: String,
    pub entries: Vec<DirEntry>,
}

impl FilePicker {
    pub fn new(path: String) -> Self {
        let real_path = expand_path(path.clone());
        FilePicker {
            path: real_path.display().to_string(),
            entries: get_entries(path.clone()),
        }
    }

    pub fn render(
        self: &Self,
        frame: &mut Frame,
        area: Rect,
        state: &mut TableState,
        theme: &Theme,
    ) {
        frame.render_widget(Clear, area);

        let rows: Vec<Row> = self
            .entries
            .iter()
            .map(|entry| {
                Row::new([icon_for(entry).to_string() + entry.file_name().to_str().take().unwrap()])
            })
            .collect();
        let widths = [Constraint::Percentage(100)];

        let block = Block::new()
            .title(self.path.clone())
            .padding(Padding::new(2, 2, 1, 1))
            .borders(Borders::all())
            .title_alignment(Alignment::Center);

        let table = Table::new(rows, widths)
            .style(Theme::color(&theme.general.foreground))
            .row_highlight_style(
                Style::default()
                    .fg(Theme::color(&theme.table.row_highlight_fg))
                    .bg(Theme::color(&theme.table.row_highlight_bg))
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .block(block);

        frame.render_stateful_widget(table, area, state);
    }
}

pub fn icon_for(entry: &DirEntry) -> &'static str {
    let path = entry.path();

    if path.is_dir() {
        return "📁";
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => "🖼️", // Image
            "mp4" | "mkv" | "avi" | "mov" | "webm" => "🎞️",          // Video
            "pdf" => "📄",                                           // PDF
            "torrent" => "🌊",                                       // Torrent
            _ => "📄",                                               // Default for other files
        }
    } else {
        "📄"
    }
}
