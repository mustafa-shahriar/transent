use crate::components::util::readable_size;
use ratatui::{Frame, layout::Rect, widgets::Paragraph};
use transmission_rpc::types::Torrent;

pub struct Details {
    pub torrent: Option<Torrent>,
}

impl Details {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match &self.torrent {
            Some(torrent) => {
                let name = torrent.name.clone().unwrap();
                let size = torrent.total_size.unwrap();
                let size = readable_size(size);
                let p = format!("Name: {}\nSize: {}", name, size);
                let p = Paragraph::new(p);
                frame.render_widget(p, area);
            }
            None => {}
        }
    }
}
