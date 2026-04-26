use crate::{
    config::Theme,
    util::{centered_rect, readable_size},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lava_torrent::torrent::v1::Torrent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState,
    },
};
use transmission_rpc::types::{Priority, TorrentAddArgs};

#[derive(Clone, Debug)]
pub struct FileEntry {
    name: String,
    size: u64,
    selected: bool,
    priority: Priority,
}

pub struct TorrentAdder {
    path: String,
    entries: Vec<FileEntry>,
    state: TableState,
    scroll_state: ScrollbarState,
    torrent_name: String,
    total_size: u64,
}

impl TorrentAdder {
    pub fn new(path: String) -> Self {
        let mut adder = Self {
            path: path.clone(),
            entries: Vec::new(),
            state: TableState::default(),
            scroll_state: ScrollbarState::default(),
            torrent_name: String::new(),
            total_size: 0,
        };
        adder.load_torrent();
        adder
    }

    pub fn handler(&mut self, key: KeyEvent) -> (bool, Option<TorrentAddArgs>) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                (false, None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.prev();
                (false, None)
            }
            KeyCode::Char(' ') => {
                self.toggle_selected();
                (false, None)
            }
            KeyCode::Char('p') => {
                self.cycle_priority();
                (false, None)
            }
            KeyCode::Char('a') => {
                self.toggle_all();
                (false, None)
            }
            KeyCode::Enter => {
                let mut t = TorrentAddArgs {
                    filename: Some(self.path.to_string()),
                    ..Default::default()
                };

                let unwanted: Vec<i32> = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| !f.selected)
                    .map(|(i, _)| i as i32)
                    .collect();
                t.files_unwanted = Some(unwanted);

                let p_low = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.priority == Priority::Low)
                    .map(|(i, _)| i as i32)
                    .collect();
                t.priority_low = Some(p_low);

                let p_norm = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.priority == Priority::Normal)
                    .map(|(i, _)| i as i32)
                    .collect();
                t.priority_normal = Some(p_norm);

                let p_high = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.priority == Priority::High)
                    .map(|(i, _)| i as i32)
                    .collect();
                t.priority_high = Some(p_high);

                (true, Some(t))
            }
            KeyCode::Char('q') => (true, None),
            KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return (true, None);
            }
            _ => (false, None),
        }
    }

    fn load_torrent(&mut self) {
        let torrent = match Torrent::read_from_file(self.path.clone()) {
            Ok(t) => t,
            Err(_) => return,
        };

        self.torrent_name = torrent.name.clone();

        if let Some(files) = &torrent.files {
            for file in files {
                let name = file
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                self.total_size += file.length as u64;
                self.entries.push(FileEntry {
                    name,
                    size: file.length as u64,
                    selected: true,
                    priority: Priority::Normal,
                });
            }
        } else {
            self.total_size = torrent.length as u64;
            self.entries.push(FileEntry {
                name: torrent.name.clone(),
                size: torrent.length as u64,
                selected: true,
                priority: Priority::Normal,
            });
        }

        if !self.entries.is_empty() {
            self.state.select(Some(0));
            self.scroll_state = ScrollbarState::new(self.entries.len().saturating_sub(1));
        }
    }

    pub fn next(&mut self) {
        match self.state.selected() {
            Some(n) if n >= self.entries.len() - 1 => self.state.select(Some(0)),
            _ => self.state.select_next(),
        }
    }

    pub fn prev(&mut self) {
        match self.state.selected() {
            Some(0) => self.state.select(Some(self.entries.len() - 1)),
            _ => self.state.select_previous(),
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(i) = self.state.selected()
            && let Some(entry) = self.entries.get_mut(i)
        {
            entry.selected = !entry.selected;
        }
    }

    pub fn cycle_priority(&mut self) {
        if let Some(i) = self.state.selected()
            && let Some(entry) = self.entries.get_mut(i)
        {
            entry.priority = match entry.priority {
                Priority::Low => Priority::Normal,
                Priority::Normal => Priority::High,
                Priority::High => Priority::Low,
            }
        }
    }

    pub fn toggle_all(&mut self) {
        let all_selected = self.entries.iter().all(|e| e.selected);
        for entry in self.entries.iter_mut() {
            entry.selected = !all_selected;
        }
    }

    pub fn render(&mut self, frame: &mut Frame, theme: &Theme) {
        let area = centered_rect(80, 66, frame.area());
        frame.render_widget(Clear, area);

        // ── outer layout: title block + table + footer ──────────────────────
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header / torrent name
                Constraint::Min(0),    // file table
                Constraint::Length(3), // footer: totals + keybinds
            ])
            .split(area);

        // ── header ──────────────────────────────────────────────────────────
        let selected_size: u64 = self
            .entries
            .iter()
            .filter(|e| e.selected)
            .map(|e| e.size)
            .sum();

        let header_block = Block::default()
            .title(format!(
                " 󰱑  {} — {} / {} ",
                self.torrent_name,
                readable_size(selected_size),
                readable_size(self.total_size),
            ))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().fg(Theme::color(&theme.general.foreground)));

        frame.render_widget(header_block, chunks[0]);

        // ── table ────────────────────────────────────────────────────────────
        let selected_idx = self.state.selected().unwrap_or(0);

        let rows: Vec<Row> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let is_focused = i == selected_idx;

                let checkbox = if entry.selected { "  " } else { "  " };

                let row_style = if is_focused {
                    Style::default()
                        .fg(Theme::color(&theme.table.row_highlight_fg))
                        .bg(Theme::color(&theme.table.row_highlight_bg))
                        .add_modifier(Modifier::BOLD)
                } else if !entry.selected {
                    Style::default()
                        .fg(Theme::color(&theme.general.foreground))
                        .add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(Theme::color(&theme.general.foreground))
                };

                Row::new([
                    Cell::from(checkbox),
                    Cell::from(entry.name.clone()),
                    Cell::from(readable_size(entry.size)),
                    Cell::from(match entry.priority {
                        Priority::Low => "Low",
                        Priority::Normal => "Normal",
                        Priority::High => "High",
                    })
                    .style(row_style),
                ])
                .style(row_style)
            })
            .collect();

        let header_row = Row::new([
            Cell::from(" Sel"),
            Cell::from("Name"),
            Cell::from("Size"),
            Cell::from("Priority"),
        ])
        .style(
            Style::default()
                .fg(Theme::color(&theme.general.foreground))
                .add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
        )
        .height(1);

        let widths = [
            Constraint::Length(5),  // checkbox
            Constraint::Min(20),    // name
            Constraint::Length(12), // size
            Constraint::Length(10), // priority
        ];

        let table_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .padding(Padding::horizontal(1))
            .style(Style::default().fg(Theme::color(&theme.general.foreground)));

        let table = Table::new(rows, widths)
            .header(header_row)
            .block(table_block)
            .row_highlight_style(
                Style::default()
                    .fg(Theme::color(&theme.table.row_highlight_fg))
                    .bg(Theme::color(&theme.table.row_highlight_bg))
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(table, chunks[1], &mut self.state);

        // ── scrollbar ────────────────────────────────────────────────────────
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼")),
            chunks[1],
            &mut self.scroll_state,
        );

        // ── footer keybinds ──────────────────────────────────────────────────
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Theme::color(&theme.general.foreground)));

        let hints = Line::from(vec![
            Span::raw(" [↑↓ navigate]  "),
            Span::raw("[Space select]  "),
            Span::raw("[p priority]  "),
            Span::raw("[a select all]  "),
            Span::raw("[Enter confirm]  "),
            Span::raw("[Esc cancel] "),
        ]);

        let footer = ratatui::widgets::Paragraph::new(hints)
            .block(footer_block)
            .alignment(Alignment::Center);

        frame.render_widget(footer, chunks[2]);
    }
}
