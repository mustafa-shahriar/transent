use crate::config::Theme;
use crate::util::calculate_match_score;
use crate::util::centered_rect;
use crate::util::expand_path;
use crate::util::fuzzy_match;
use crate::util::get_entries;
use crate::util::icon_for;
use crate::widgets::input::Input;
use crate::widgets::input::InputMode;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
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

pub struct FilePicker {
    path: String,
    entries: Vec<DirEntry>,
    prev_states: Vec<TableState>,
    state: TableState,

    show_hidden: bool,
    input: Input,
}

impl FilePicker {
    pub fn new(path: String, show_hidden: bool) -> Self {
        let real_path = expand_path(path.clone());
        FilePicker {
            path: real_path.display().to_string(),
            entries: get_entries(path.clone(), show_hidden),
            prev_states: vec![],
            state: TableState::default(),
            input: Input::new(),
            show_hidden,
        }
    }

    fn select_best_match(&mut self) {
        if !self.input.is_active {
            return;
        }

        let input = self.input.input.to_lowercase();

        if input.is_empty() {
            self.state.select(Some(0));
            return;
        }

        // Find the best matching entry based on fuzzy match
        let mut best_index: Option<usize> = None;
        let mut best_score = 0;

        for (index, entry) in self.entries.iter().enumerate() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if fuzzy_match(&name, &input) {
                // Score based on how early the match starts and match length
                let score = calculate_match_score(&name, &input);
                if score > best_score {
                    best_score = score;
                    best_index = Some(index);
                }
            }
        }

        if let Some(index) = best_index {
            self.state.select(Some(index));
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
            Some(0) => self.state.select(Some(self.entries.len() - 1)),
            _ => self.state.select_previous(),
        }
    }

    fn go_back(&mut self) {
        self.input.input = "".to_string();
        let path = Path::new(&self.path).to_path_buf();

        if let Some(parent) = path.parent() {
            self.path = parent.display().to_string();
            self.entries = get_entries(self.path.clone(), false);

            if !self.prev_states.is_empty() {
                self.state = self.prev_states.pop().unwrap();
            }

            let index = self
                .entries
                .iter()
                .position(|el| el.path().to_str().unwrap() == path.to_str().unwrap());

            self.state.select(index);
        }
    }

    async fn select_entry(&mut self) -> (bool, Option<String>) {
        self.input.input = "".to_string();
        match self.state.selected() {
            Some(n) if n < self.entries.len() => {
                let path = self.entries[n]
                    .path()
                    .canonicalize()
                    .unwrap()
                    .display()
                    .to_string();

                if path.ends_with(".torrent") {
                    return (true, Some(path));
                }

                self.prev_states.push(self.state.clone());
                self.path = path;
                self.entries = get_entries(self.path.to_string(), self.show_hidden);
                match self.entries.len() {
                    0 => self.state.select(None),
                    _ => self.state.select(Some(0)),
                }
            }
            None => {}
            _ => {}
        }
        (false, None)
    }

    pub async fn handler(&mut self, key: KeyEvent) -> (bool, Option<String>) {
        if self.input.is_active {
            self.input.handler(key);
            self.select_best_match();
            return (false, None);
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('h') | KeyCode::Right => self.go_back(),
            KeyCode::Char('l') | KeyCode::Enter | KeyCode::Left => {
                return self.select_entry().await;
            }
            KeyCode::Char('/') => {
                self.input.is_active = true;
                self.input.input_mode = InputMode::Editing;
                self.input.reset_cursor();
            }
            KeyCode::Char('q') | KeyCode::Esc => return (true, None),
            _ => {}
        }
        (false, None)
    }

    pub fn render(&mut self, frame: &mut Frame, theme: &Theme) {
        let area = centered_rect(50, 75, frame.area());

        let rows: Vec<Row> = self
            .entries
            .iter()
            .map(|entry| {
                Row::new([icon_for(entry).to_string() + entry.file_name().to_str().unwrap()])
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Percentage(80), Percentage(20)])
            .split(area);

        if self.input.is_active {
            frame.render_widget(Clear, area);
            frame.render_stateful_widget(table, chunks[0], &mut self.state);
            self.input.render(frame, chunks[1], theme);
        } else {
            frame.render_widget(Clear, chunks[0]);
            frame.render_stateful_widget(table, chunks[0], &mut self.state);
        }
    }
}
