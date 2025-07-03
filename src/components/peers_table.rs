use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Table, TableState};
use ratatui::{Frame, layout::Rect, widgets::Row};
use transmission_rpc::types::Torrent;

use crate::components::util::readble_speed;
use crate::theme::Theme;

pub fn render_peers_table(
    torrent: Torrent,
    frame: &mut Frame,
    area: Rect,
    table_state: &mut TableState,
    theme: &Theme,
) {
    let header =
        Row::new(["Adress", "Client", "Download Speed", "Upload Speed"]).style(Style::new().bold());

    let rows: Vec<Row> = torrent
        .peers
        .unwrap()
        .iter()
        .map(|peer| {
            Row::new([
                peer.address.to_string().clone(),
                peer.client_name.clone(),
                readble_speed((peer.rate_to_client / (8 * 1024)) as i64),
                readble_speed((peer.rate_to_peer / (8 * 1024)) as i64),
            ])
        })
        .collect();
    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(40),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
    ];

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
