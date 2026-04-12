use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;
use ratatui::widgets::Widget;
use transmission_rpc::types::Id;

static ACTIONS_STR: [&str; 4] = ["Pause", "Resume", "Delete", "Delete including Data"];

pub struct TorrentActions {
    pub id: Id,
    pub name: String,
    state: TableState,
}

impl TorrentActions {
    pub fn new(id: Id, name: String) -> Self {
        Self {
            id,
            name,
            state: TableState::default(),
        }
    }

    pub fn select_next(&mut self) {
        match self.state.selected() {
            Some(n) if n >= ACTIONS_STR.len() - 1 => self.state.select(Some(0)),
            _ => self.state.select_next(),
        }
    }

    pub fn select_prev(&mut self) {
        match self.state.selected() {
            Some(n) if n <= 0 => self.state.select(Some(ACTIONS_STR.len() - 1)),
            _ => self.state.select_previous(),
        }
    }

    pub fn get_selected(&self) -> Option<&str> {
        let index = self.state.selected();
        match index {
            Some(i) => Some(ACTIONS_STR[i]),
            None => None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, theme: &Theme) {
        let area = Rect {
            x: frame.area().width / 4,
            y: frame.area().height / 3,
            width: frame.area().width / 2,
            height: frame.area().height / 2,
        };
        Clear.render(area, frame.buffer_mut());
        let rows: Vec<Row> = ACTIONS_STR
            .iter()
            .map(|action| Row::new([*action]))
            .collect();

        let widths = [Constraint::Percentage(100)];

        let table = Table::new(rows, widths)
            .column_spacing(2)
            .style(Style::default().fg(Theme::color(&theme.general.foreground)))
            .row_highlight_style(
                Style::default()
                    .fg(Theme::color(&theme.table.row_highlight_fg))
                    .bg(Theme::color(&theme.table.row_highlight_bg))
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Theme::color(&theme.general.foreground)))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .padding(ratatui::widgets::Padding::uniform(1))
                    .title(self.name.clone()),
            );

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}
