use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, TableState};
use transmission_rpc::types::TorrentSetArgs;
use transmission_rpc::types::{File, Priority};

use crate::theme::Theme;
use crate::util::readable_size;

const BAR_WIDTH: usize = 20;
const WIDTHS: [Constraint; 5] = [
    Constraint::Length(3),                // 0 checkbox
    Constraint::Percentage(50),           // 1 name
    Constraint::Length(10),               // 2 size
    Constraint::Length(BAR_WIDTH as u16), // 3 progress bar
    Constraint::Percentage(15),           // 4 priority
];

pub struct FilesTable {
    pub files: Vec<File>,
    pub priorities: Vec<Priority>,
    pub wanted: Vec<bool>,
    state: TableState,
}

impl FilesTable {
    pub fn new() -> Self {
        Self {
            files: vec![],
            priorities: vec![],
            wanted: vec![],
            state: TableState::default(),
        }
    }

    // ── Navigation ────────────────────────────────────────────────────────────

    pub fn select_next(&mut self) {
        match self.state.selected() {
            Some(n) if n >= self.files.len() - 1 => self.state.select(Some(0)),
            _ => self.state.select_next(),
        }
    }

    pub fn select_prev(&mut self) {
        match self.state.selected() {
            Some(n) if n <= 0 => self.state.select(Some(self.files.len() - 1)),
            _ => self.state.select_previous(),
        }
    }

    pub fn select_first(&mut self) {
        if !self.files.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if !self.files.is_empty() {
            self.state.select(Some(self.files.len() - 1));
        }
    }

    // ── Actions ───────────────────────────────────────────────────────────────

    pub fn toggle_wanted(&mut self) -> Option<TorrentSetArgs> {
        if let Some(i) = self.state.selected() {
            if let Some(w) = self.wanted.get(i) {
                let tsa = TorrentSetArgs::new();
                if *w {
                    return Some(tsa.files_unwanted(vec![i]));
                } else {
                    return Some(tsa.files_wanted(vec![i]));
                }
            }
        }
        None
    }

    pub fn cycle_priority(&mut self) -> Option<TorrentSetArgs> {
        if let Some(i) = self.state.selected() {
            if let Some(p) = self.priorities.get_mut(i) {
                let tsa = TorrentSetArgs::new();
                let tsa = match p {
                    Priority::Low => tsa.priority_normal(vec![i]),
                    Priority::Normal => tsa.priority_high(vec![i]),
                    Priority::High => tsa.priority_low(vec![i]),
                };
                return Some(tsa);
            }
        }
        None
    }

    // ── Key handler ───────────────────────────────────────────────────────────

    pub fn handler(&mut self, key: KeyEvent) -> Option<TorrentSetArgs> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                None
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.select_first();
                None
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.select_last();
                None
            }
            KeyCode::Char(' ') => {
                return self.toggle_wanted();
            }
            KeyCode::Char('p') => {
                return self.cycle_priority();
            }
            _ => None,
        }
    }

    // ── Render ────────────────────────────────────────────────────────────────

    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let header =
            Row::new(["  ", "Name", "Size", "Progress", "Priority"]).style(Style::new().bold());

        let rows: Vec<Row> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let wanted = self.wanted.get(i).copied().unwrap_or(true);
                let checkbox = if wanted { "" } else { "" };
                let size = readable_size(file.length as u64);
                let priority = self
                    .priorities
                    .get(i)
                    .map(|p| match p {
                        Priority::Low => "Low",
                        Priority::Normal => "Normal",
                        Priority::High => "High",
                    })
                    .unwrap_or("N/A");

                let progress_cell = Cell::from(progress_bar(
                    file.bytes_completed as u64,
                    file.length as u64,
                    Theme::color(&theme.progress_bar.filled),
                    Theme::color(&theme.progress_bar.empty),
                ));

                let name: Vec<&str> = file.name.split_terminator('/').collect();
                let name = name.get(name.len() - 1).unwrap().to_string();

                let row = Row::new([
                    Cell::from(checkbox),
                    Cell::from(name),
                    Cell::from(size),
                    progress_cell,
                    Cell::from(priority),
                ]);

                if wanted {
                    row
                } else {
                    row.style(Style::default().dim())
                }
            })
            .collect();

        let table = Table::new(rows, WIDTHS)
            .header(header)
            .column_spacing(1)
            .style(Theme::color(&theme.general.foreground))
            .row_highlight_style(
                Style::default()
                    .fg(Theme::color(&theme.table.row_highlight_fg))
                    .bg(Theme::color(&theme.table.row_highlight_bg))
                    .add_modifier(ratatui::style::Modifier::BOLD),
            );

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}

// ── Progress bar ──────────────────────────────────────────────────────────────
//
// Returns a `Line` made of two `Span`s:
//
//   [  filled bg  |  label chars  |  empty bg  ]
//
// The label (e.g. " 57%") is centered across the full bar width.
// Characters that fall inside the filled region get the filled background;
// characters that fall inside the empty region get the empty background.
// This means the label text itself "bleeds through" the color boundary —
// exactly like ratatui's own Gauge widget.
//
// Visual example (BAR_WIDTH = 20, 57%):
//   ████████████ 57%░░░░░   (blue bg left, dark bg right)

fn progress_bar(completed: u64, total: u64, filled_bg: Color, empty_bg: Color) -> Line<'static> {
    let pct = if total > 0 {
        ((completed as f64 / total as f64) * 100.0).round() as usize
    } else {
        0
    }
    .min(100);

    let filled_chars = pct * BAR_WIDTH / 100;

    // Center the label inside the bar.
    let label = format!("{pct}%");
    // Full-width string: spaces + label + spaces, exactly BAR_WIDTH chars.
    let bar_text: String = format!("{:^width$}", label, width = BAR_WIDTH);

    let filled_style = Style::reset().fg(Color::White).bg(filled_bg);
    let empty_style = Style::reset().fg(Color::DarkGray).bg(empty_bg);

    if filled_chars == 0 {
        // Entirely empty.
        Line::from(Span::styled(bar_text, empty_style))
    } else if filled_chars >= BAR_WIDTH {
        // Entirely filled.
        Line::from(Span::styled(bar_text, filled_style))
    } else {
        // Split: first `filled_chars` chars use filled_style, the rest empty_style.
        let filled_str: String = bar_text.chars().take(filled_chars).collect();
        let empty_str: String = bar_text.chars().skip(filled_chars).collect();
        Line::from(vec![
            Span::styled(filled_str, filled_style),
            Span::styled(empty_str, empty_style),
        ])
    }
}
