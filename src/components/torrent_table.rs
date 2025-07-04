use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Table, TableState};
use ratatui::{Frame, layout::Rect, widgets::Row};
use transmission_rpc::types::Torrent;

use crate::components::util::{readabl_eta, readble_speed, status_to_string};
use crate::theme::Theme;

pub struct TorrentTable<'a> {
    pub torrents: &'a [Torrent],
}

impl<'a> TorrentTable<'a> {
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        table_state: &mut TableState,
        theme: &Theme,
    ) {
        let header = Row::new([
            "Name",
            "Status",
            "Progress",
            "Download Speed",
            "Upload Speed",
            "ETA",
        ])
        .style(Style::new().bold());

        let rows: Vec<Row> = self
            .torrents
            .iter()
            .map(|torrent| {
                let status = status_to_string(torrent.status.unwrap());
                let progress = torrent.percent_done.unwrap_or(0.0) * 100.0;
                let down = torrent.rate_download.unwrap_or(0);
                let up = torrent.rate_upload.unwrap_or(0);
                let eta = torrent.eta.unwrap_or(0);
                Row::new([
                    torrent.name.clone().unwrap_or_default(),
                    status,
                    format!("{:.1}%", progress),
                    readble_speed(down),
                    readble_speed(up),
                    readabl_eta(eta),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(50),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
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
}
