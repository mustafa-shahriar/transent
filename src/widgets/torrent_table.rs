use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Row;
use ratatui::widgets::Scrollbar;
use ratatui::widgets::ScrollbarOrientation;
use ratatui::widgets::ScrollbarState;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;
use transmission_rpc::types::Torrent;

use crate::theme::Theme;
use crate::util::readabl_eta;
use crate::util::readble_speed;
use crate::util::status_to_string;

pub struct TorrentTable {
    pub torrents: Vec<Torrent>,
    pub state: TableState,
    pub scrollbar_state: ScrollbarState,
}

impl TorrentTable {
    pub fn select_next(&mut self) {
        match self.state.selected() {
            Some(n) if n >= self.torrents.len() - 1 => self.state.select(Some(0)),
            _ => self.state.select_next(),
        }
    }

    pub fn select_prev(&mut self) {
        match self.state.selected() {
            Some(0) => self.state.select(Some(self.torrents.len() - 1)),
            _ => self.state.select_previous(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
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

        frame.render_stateful_widget(table, container[0], &mut self.state);
        frame.render_stateful_widget(scrollbar, container[1], &mut self.scrollbar_state);
    }
}
