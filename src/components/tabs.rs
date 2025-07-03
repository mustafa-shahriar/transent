use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::{
    Frame,
    widgets::{Block, Borders, Tabs},
};

pub fn render_tabs<'a, T: ToString>(
    titles: &[T],
    selected: usize,
    area: Rect,
    frame: &mut Frame,
    focused: bool,
    theme: &Theme,
) {
    use ratatui::style::{Modifier, Style};
    let titles: Vec<_> = titles.iter().map(|t| t.to_string()).collect();
    let base_style = Style::default()
        .fg(Theme::color(&theme.tabs.inactive_fg))
        .bg(Theme::color(&theme.tabs.inactive_bg));
    let highlight_style = if focused {
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
    let tabs = Tabs::new(titles)
        .select(selected)
        .block(Block::default().borders(Borders::ALL))
        .style(base_style)
        .highlight_style(highlight_style);
    frame.render_widget(tabs, area);
}
