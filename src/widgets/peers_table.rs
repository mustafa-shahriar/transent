use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::Padding;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;
use transmission_rpc::types::Peer;

use crate::theme::Theme;
use crate::util::readble_speed;

pub struct PeersTable {
    pub peers: Vec<Peer>,
    state: TableState,
}

impl PeersTable {
    pub fn new() -> Self {
        Self {
            peers: vec![],
            state: TableState::default(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let header = Row::new(["Adress", "Client", "Download Speed", "Upload Speed"])
            .style(Style::new().bold());

        let rows: Vec<Row> = self
            .peers
            .iter()
            .map(|peer| {
                Row::new([
                    peer.address.to_string().clone(),
                    peer.client_name.clone(),
                    readble_speed((peer.rate_to_client) as i64),
                    readble_speed((peer.rate_to_peer) as i64),
                ])
            })
            .collect();
        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ];

        let block = Block::default().padding(Padding::new(1, 1, 0, 0));
        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(2)
            .block(block)
            .style(Theme::color(&theme.general.foreground))
            .row_highlight_style(
                Style::default()
                    .fg(Theme::color(&theme.table.row_highlight_fg))
                    .bg(Theme::color(&theme.table.row_highlight_bg))
                    .add_modifier(ratatui::style::Modifier::BOLD),
            );

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}
