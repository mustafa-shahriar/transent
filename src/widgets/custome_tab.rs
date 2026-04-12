use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Tabs;
use ratatui::style::Modifier;
use ratatui::style::Style;

pub struct CustomeTabs {
    titles: Vec<String>,
    selected: usize,
    pub is_focused: bool,
}

impl CustomeTabs {
    pub fn new(titles: Vec<String>, is_focused: bool) -> Self {
        Self {
            titles,
            selected: 0,
            is_focused,
        }
    }

    pub fn select_next(&mut self) {
        self.selected += 1;
        if self.selected == self.titles.len() {
            self.selected = 0;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected == 0 {
            self.selected = self.titles.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn selected_tab(&self) -> String {
        self.titles[self.selected].clone()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let base_style = Style::default()
            .fg(Theme::color(&theme.tabs.inactive_fg))
            .bg(Theme::color(&theme.tabs.inactive_bg));

        let highlight_style = if self.is_focused {
            Style::default()
                .fg(Theme::color(&theme.tabs.active_fg))
                .bg(Theme::color(&theme.tabs.active_bg))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default()
                .fg(Theme::color(&theme.tabs.inactive_fg))
                .bg(Theme::color(&theme.tabs.inactive_bg))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        };

        let tabs = Tabs::new(self.titles.clone())
            .select(self.selected)
            .block(Block::default().borders(Borders::ALL))
            .style(base_style)
            .highlight_style(highlight_style);

        frame.render_widget(tabs, area);
    }
}
