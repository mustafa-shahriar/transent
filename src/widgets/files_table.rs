use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;
use transmission_rpc::types::File;
use transmission_rpc::types::Priority;

use crate::theme::Theme;

pub struct FilesTable {
    pub files: Vec<File>,
    pub priorities: Vec<Priority>,
    state: TableState,
}

impl FilesTable {
    pub fn new() -> Self {
        Self {
            files: vec![],
            priorities: vec![],
            state: TableState::default(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let header = Row::new(["Name", "Priority"]).style(Style::new().bold());

        let rows: Vec<Row> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let priority = self
                    .priorities
                    .get(i)
                    .map(|p| match p {
                        Priority::Low => "Low",
                        Priority::Normal => "Normal",
                        Priority::High => "High",
                    })
                    .unwrap_or("N/A");

                Row::new([file.name.clone(), priority.to_string()])
            })
            .collect();

        let widths = [Constraint::Percentage(70), Constraint::Percentage(30)];

        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(2)
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
