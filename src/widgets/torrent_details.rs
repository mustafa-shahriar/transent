use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use transmission_rpc::types::Torrent;

use crate::theme::Theme;
use crate::util::readabl_eta;
use crate::util::readable_size;
use crate::util::readble_speed;
use crate::util::status_to_string;

pub struct Details {
    pub torrent: Option<Torrent>,
}

impl Details {
    pub fn new() -> Self {
        Self { torrent: None }
    }
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        match &self.torrent {
            Some(torrent) => {
                let name = torrent.name.clone().unwrap_or_default();
                let total_size = readable_size(torrent.total_size.unwrap_or(0));
                let downloaded = readable_size(torrent.downloaded_ever.unwrap_or(0) as i64);
                let uploaded = readable_size(torrent.uploaded_ever.unwrap_or(0));
                let status = status_to_string(torrent.status.unwrap());
                let progress = torrent.percent_done.unwrap_or(0.0) * 100.0;
                let progress_str = format!("{progress:.1}%");
                let down_speed = readble_speed(torrent.rate_download.unwrap_or(0));
                let up_speed = readble_speed(torrent.rate_upload.unwrap_or(0));
                let eta = readabl_eta(torrent.eta.unwrap_or(-1));
                let peers = torrent.peers_connected.unwrap_or(0);
                let peers_str = format!("{peers}");
                let seeds = torrent.peers_getting_from_us.unwrap_or(0);
                let seeds_str = format!("{seeds}");

                let base_style = Style::default()
                    .fg(Theme::color(&theme.general.foreground))
                    .bg(Theme::color(&theme.general.background));

                let alt_row_style = Style::default()
                    .bg(Theme::color(&theme.table.row_highlight_fg))
                    .fg(Theme::color(&theme.table.row_highlight_bg));

                let rows = vec![
                    Row::new(vec![Cell::from("Name"), Cell::from(name.as_str())]).style(alt_row_style),
                    Row::new(vec![Cell::from("Status"), Cell::from(status.as_str())])
                        .style(base_style),
                    Row::new(vec![
                        Cell::from("Progress"),
                        Cell::from(progress_str.as_str()),
                    ])
                    .style(alt_row_style),
                    Row::new(vec![
                        Cell::from("Total Size"),
                        Cell::from(total_size.as_str()),
                    ])
                    .style(base_style),
                    Row::new(vec![
                        Cell::from("Downloaded"),
                        Cell::from(downloaded.as_str()),
                    ])
                    .style(alt_row_style),
                    Row::new(vec![Cell::from("Uploaded"), Cell::from(uploaded.as_str())])
                        .style(base_style),
                    Row::new(vec![
                        Cell::from("Download Speed"),
                        Cell::from(down_speed.as_str()),
                    ])
                    .style(alt_row_style),
                    Row::new(vec![
                        Cell::from("Upload Speed"),
                        Cell::from(up_speed.as_str()),
                    ])
                    .style(base_style),
                    Row::new(vec![Cell::from("ETA"), Cell::from(eta.as_str())]).style(alt_row_style),
                    Row::new(vec![
                        Cell::from("Connected Peers"),
                        Cell::from(peers_str.as_str()),
                    ])
                    .style(base_style),
                    Row::new(vec![Cell::from("Seeds"), Cell::from(seeds_str.as_str())])
                        .style(alt_row_style),
                ];

                let widths = vec![Constraint::Percentage(25), Constraint::Percentage(75)];

                let table = Table::new(rows, widths)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Torrent Details")
                            .border_style(base_style),
                    )
                    .style(base_style);

                frame.render_widget(table, area);
            }
            None => {
                let p = Paragraph::new("No torrent selected");
                frame.render_widget(p, area);
            }
        }
    }
}
