use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Alignment,
    widgets::{Block, Borders, Clear, Padding},
};
use transmission_rpc::types::TorrentAddArgs;

use crate::{config::Theme, util::centered_rect, widgets::input::Input};

pub struct Magnet {
    input: Input,
}

impl Magnet {
    pub fn new() -> Self {
        let mut input = Input::new();
        input.is_active = true;
        Self { input }
    }
    pub fn render(&self, frame: &mut Frame, theme: &Theme) {
        let area = centered_rect(50, 40, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::new()
            .title("Add torrent from Magnet Link")
            .padding(Padding::new(1, 1, 3, 3))
            .borders(Borders::all())
            .title_alignment(Alignment::Center);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        self.input.render(frame, inner_area, theme);
    }

    pub fn handler(&mut self, key: KeyEvent) -> (bool, Option<TorrentAddArgs>) {
        let arg = if let Some(s) = self.input.handler(key) {
            let arg = TorrentAddArgs {
                filename: Some(s),
                files_unwanted: None,
                ..Default::default()
            };
            Some(arg)
        } else {
            None
        };

        (!self.input.is_active, arg)
    }
}

