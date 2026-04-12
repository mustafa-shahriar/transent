use crate::theme::Theme;
use crate::util::centered_rect;
use crate::util::expand_path;
use crate::util::get_entries;

use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Padding;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;
use std::fs::DirEntry;

pub struct FilePicker {
    pub path: String,
    pub entries: Vec<DirEntry>,
    pub prev_states: Vec<TableState>,
    state: TableState,

    pub show_hidden: bool,
    pub search_string: Option<String>,
}

impl FilePicker {
    pub fn new(path: String, show_hidden: bool) -> Self {
        let real_path = expand_path(path.clone());
        FilePicker {
            path: real_path.display().to_string(),
            entries: get_entries(path.clone(), show_hidden),
            prev_states: vec![],
            state: TableState::default(),
            search_string: None,
            show_hidden,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, theme: &Theme) {
        let area = centered_rect(50, 66, frame.area());
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

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}

pub fn icon_for(entry: &DirEntry) -> &'static str {
    let path = entry.path();

    if path.is_dir() {
        return "📁";
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "torrent" => "🌊", // Torrent
            _ => "📄",         // Default for other files
        }
    } else {
        "📄"
    }
}
