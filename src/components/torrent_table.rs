use ratatui::layout::Constraint;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Table, TableState};
use ratatui::{Frame, layout::Rect, widgets::Row};
use transmission_rpc::types::Torrent;
use transmission_rpc::types::TorrentStatus;

pub struct TorrentTable {
    pub torrents: Vec<Torrent>,
}

impl TorrentTable {
    pub fn render(&self, frame: &mut Frame, area: Rect, table_state: &mut TableState) {
        let header = Row::new([
            "Name",
            "Status",
            "Progress",
            "Download Speed",
            "Upload Speed",
            "ETA",
        ])
        .style(Style::new().bold());

        let mut rows: Vec<Row> = vec![];
        for torrent in self.torrents.iter() {
            let status = status_to_string(torrent.status.unwrap());
            let porgress = torrent.percent_done.unwrap_or(0.0) * 100.0;
            let down = torrent.rate_download.unwrap_or(0).to_string();
            let up = torrent.rate_upload.unwrap_or(0).to_string();
            let eta = torrent.eta.unwrap_or(0).to_string();
            let row = [
                torrent.name.clone().unwrap(),
                status.to_string(),
                porgress.to_string() + "%",
                down,
                up,
                eta,
            ];
            rows.push(Row::new(row));
        }

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
            .style(Color::White)
            .row_highlight_style(Style::new().on_black().cyan());

        frame.render_stateful_widget(table, area, table_state);
    }
}

fn status_to_string(status: TorrentStatus) -> String {
    let s = match status {
        TorrentStatus::Stopped => "Stopped",
        TorrentStatus::QueuedToVerify => "QueuedToVerify",
        TorrentStatus::Verifying => "Verifying",
        TorrentStatus::QueuedToDownload => "QueuedToDownload",
        TorrentStatus::Downloading => "Downloading",
        TorrentStatus::QueuedToSeed => "QueuedToSeed",
        TorrentStatus::Seeding => "Seeding",
    };

    return s.to_string();
}
