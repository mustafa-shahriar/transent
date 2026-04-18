use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use transmission_rpc::types::Id;

use crate::{theme::Theme, util::centered_rect};

pub struct DeletePopup {
    pub id: Id,
    pub name: String,
    pub with_data: bool,
}

impl DeletePopup {
    pub fn new(id: Id, name: String, with_data: bool) -> Self {
        Self {
            id,
            name,
            with_data,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, theme: &Theme) {
        let area = centered_rect(50, 50, frame.area());

        frame.render_widget(Clear, area);

        let text = if self.with_data {
            "Delete With Data"
        } else {
            "Delete Without Data"
        };

        let paragraph = Paragraph::new(format!("\n\n{}\n\n[Y]es    [N]o", text))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Theme::color(&theme.general.foreground)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.name.clone())
                    .border_type(BorderType::Rounded),
            );

        frame.render_widget(paragraph, area);
    }
}
