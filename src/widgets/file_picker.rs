use crate::theme::Theme;
use crate::util::centered_rect;
use crate::util::expand_path;
use crate::util::get_entries;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
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
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use transmission_rpc::types::TorrentAddArgs;

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

    fn select_next(&mut self) {
        match self.state.selected() {
            Some(n) if n >= self.entries.len() - 1 => self.state.select(Some(0)),
            _ => self.state.select_next(),
        }
    }

    fn select_prev(&mut self) {
        match self.state.selected() {
            Some(n) if n <= 0 => self.state.select(Some(self.entries.len() - 1)),
            _ => self.state.select_previous(),
        }
    }

    fn go_back(&mut self) {
        let path = Path::new(&self.path);
        match path.parent() {
            Some(parent) => {
                self.path = parent.display().to_string();
                self.entries = get_entries(self.path.clone(), false);
                if self.prev_states.len() != 0 {
                    self.state = self.prev_states.pop().unwrap();
                }
            }
            None => {}
        }
    }

    async fn select_entry(&mut self, client: &Arc<Mutex<transmission_rpc::TransClient>>) -> bool {
        match self.state.selected() {
            Some(n) => {
                self.prev_states.push(self.state.clone());
                let selected_path = self.entries[n]
                    .path()
                    .canonicalize()
                    .unwrap()
                    .display()
                    .to_string();
                self.path = selected_path;

                if self.path.ends_with(".torrent") {
                    let mut t = TorrentAddArgs::default();
                    t.files_unwanted = None;
                    t.filename = Some(self.path.clone());
                    let r = client.lock().await.torrent_add(t).await;
                    match r {
                        Ok(_) => return true,
                        Err(_) => {}
                    }
                    return false;
                };

                self.entries = get_entries(self.path.to_string(), false);
                match self.entries.len() {
                    0 => self.state.select(None),
                    _ => self.state.select(Some(0)),
                }
            }
            None => {}
        }
        false
    }

    pub async fn handler(
        &mut self,
        key: KeyEvent,
        client: &Arc<Mutex<transmission_rpc::TransClient>>,
    ) -> bool {
        match key.code {
            KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('k') => self.select_prev(),
            KeyCode::Char('h') => self.go_back(),
            KeyCode::Char('l') => return self.select_entry(client).await,
            _ => {}
        }
        false
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
