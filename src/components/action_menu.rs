use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
};

use crate::theme::Theme;

static ACTIONS_STR: [&str; 4] = ["Pause", "Resume", "Delete", "Delete including Data"];

pub fn render_actions(frame: &mut Frame, area: Rect, index: usize, theme: &Theme) {
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
        // wrap the table in a block with borders and margin
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Theme::color(&theme.general.foreground)))
                .border_type(ratatui::widgets::BorderType::Rounded) // optional: Rounded, Plain, Double, Thick
                .padding(ratatui::widgets::Padding::uniform(1)), // margin inside the block
        );

    let mut state = TableState::default();
    state.select(Some(index));

    frame.render_stateful_widget(table, area, &mut state);
}
