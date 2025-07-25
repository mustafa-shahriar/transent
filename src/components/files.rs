use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Table, TableState};
use ratatui::{Frame, layout::Rect, widgets::Row};
use transmission_rpc::types::{Priority, Torrent};

use crate::theme::Theme;

pub fn render_files_tab(
    torrent: Torrent,
    frame: &mut Frame,
    area: Rect,
    table_state: &mut TableState,
    theme: &Theme,
) {
    let header = Row::new(["Name", "Priority"]).style(Style::new().bold());

    let files = torrent.files.unwrap_or_default();
    let priorities = torrent.priorities.unwrap_or_default();

    let rows: Vec<Row> = files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let priority = priorities
                .get(i)
                .map(|p| match p {
                    Priority::Low => "Low",
                    Priority::Normal => "Nornal",
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

    frame.render_stateful_widget(table, area, table_state);
}
