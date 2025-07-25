use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
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
        scrollbar_state: &mut ScrollbarState,
        theme: &Theme,
    ) {
        let container = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(95), Constraint::Percentage(5)])
            .split(area);
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

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(ratatui::symbols::scrollbar::VERTICAL)
            .begin_symbol(Some(""))
            .end_symbol(Some(""))
            .thumb_style(
                Style::default()
                    .fg(Theme::color(&theme.general.foreground))
                    .bg(Theme::color(&theme.general.background))
                    .add_modifier(ratatui::style::Modifier::BOLD),
            );

        frame.render_stateful_widget(table, container[0], table_state);
        frame.render_stateful_widget(scrollbar, container[1], scrollbar_state);
    }
}
