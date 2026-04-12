use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use transmission_rpc::types::Torrent;

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
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match &self.torrent {
            Some(torrent) => {
                let name = torrent.name.clone().unwrap_or_default();
                let total_size = readable_size(torrent.total_size.unwrap_or(0));
                let downloaded = readable_size(torrent.downloaded_ever.unwrap_or(0) as i64);
                let uploaded = readable_size(torrent.uploaded_ever.unwrap_or(0));
                let status = status_to_string(torrent.status.unwrap());
                let progress = torrent.percent_done.unwrap_or(0.0) * 100.0;
                let down_speed = readble_speed(torrent.rate_download.unwrap_or(0));
                let up_speed = readble_speed(torrent.rate_upload.unwrap_or(0));
                let eta = readabl_eta(torrent.eta.unwrap_or(-1));
                let peers = torrent.peers_connected.unwrap_or(0);
                let seeds = torrent.peers_getting_from_us.unwrap_or(0);

                let details = format!(
                    "Name: {}\n\
                     Status: {}\n\
                     Progress: {:.1}%\n\
                     Total Size: {}\n\
                     Downloaded: {}\n\
                     Uploaded: {}\n\
                     Download Speed: {}\n\
                     Upload Speed: {}\n\
                     ETA: {}\n\
                     Connected Peers: {}\n\
                     Seeds: {}",
                    name,
                    status,
                    progress,
                    total_size,
                    downloaded,
                    uploaded,
                    down_speed,
                    up_speed,
                    eta,
                    peers,
                    seeds
                );
                let p = Paragraph::new(details);
                frame.render_widget(p, area);
            }
            None => {}
        }
    }
}
