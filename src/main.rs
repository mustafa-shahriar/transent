use color_eyre::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    widgets::{Block, Borders, TableState, Tabs},
};
use transmission_rpc::{
    TransClient,
    types::{Torrent, TorrentStatus},
};
mod components;
use components::torrent_table::TorrentTable;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use url::Url;

use crate::{components::details, key_config::load_keymap};
mod key_config;
mod theme;
use key_config::{Actions, keyevent_to_string};
use std::fs;
use theme::Theme;

fn load_theme() -> Theme {
    let content = fs::read_to_string("theme.toml").expect("theme.toml not found");
    toml::from_str(&content).expect("Invalid theme.toml")
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tokio_main()
}

#[tokio::main]
async fn tokio_main() -> color_eyre::Result<()> {
    let url = Url::parse(
        "http://mustafa:0392d1666f6043b99ee129697376cd1b7b20394f53dnsPb6@0.0.0.0:9091/transmission/rpc",
    )?;
    let client = Arc::new(Mutex::new(TransClient::new(url)));

    // Initial fetch
    let torrents = client
        .lock()
        .await
        .torrent_get(None, None)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e))?
        .arguments
        .torrents;

    let torrents_arc = Arc::new(Mutex::new(torrents));

    // Clone for background task
    let torrents_bg = torrents_arc.clone();
    let client_bg = client.clone();

    // Spawn background updater
    tokio::spawn(async move {
        loop {
            let mut client = client_bg.lock().await;
            match client.torrent_get(None, None).await {
                Ok(resp) => {
                    let mut torrents = torrents_bg.lock().await;
                    *torrents = resp.arguments.torrents;
                }
                Err(e) => eprintln!("Error fetching torrents: {e}"),
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    let app = App::new(torrents_arc);

    let terminal = ratatui::init();
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Pane {
    Top,
    Bottom,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum TopTab {
    All,
    Completed,
    Downloading,
    Seeding,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum BottomTab {
    Details,
    Peers,
    Files,
}

/// The main application which holds the state and logic of the application.
pub struct App {
    running: bool,
    torrents: Arc<Mutex<Vec<Torrent>>>,
    pub table_state: TableState,
    pub peer_table_state: TableState,
    pub file_table_state: TableState,
    focused_pane: Pane,
    top_tab: TopTab,
    bottom_tab: BottomTab,
    pub theme: Theme,
    pub key_map: HashMap<String, Actions>,
}

impl App {
    pub fn new(torrents: Arc<Mutex<Vec<Torrent>>>) -> Self {
        let key_map = load_keymap("key_config.toml");
        App {
            running: true,
            torrents,
            table_state: TableState::default(),
            peer_table_state: TableState::default(),
            file_table_state: TableState::default(),
            focused_pane: Pane::Top,
            top_tab: TopTab::All,
            bottom_tab: BottomTab::Details,
            theme: load_theme(),
            key_map: key_map,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            // Lock only for the duration of rendering
            let torrents = { self.torrents.lock().await.clone() }; // clone the Vec<Torrent>
            terminal.draw(|frame| self.render(frame, &torrents))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, torrents: &[Torrent]) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),      // Top tabs
                Constraint::Percentage(47), // Top pane
                Constraint::Length(3),      // Bottom tabs
                Constraint::Percentage(47), // Bottom pane
            ])
            .split(frame.area());

        // Top tabs
        let top_tab_titles = ["All", "Completed", "Downloading", "Seeding"];
        render_tabs(
            &top_tab_titles,
            self.top_tab as usize,
            chunks[0],
            frame,
            self.focused_pane == Pane::Top,
            &self.theme,
        );

        // Filter torrents for top tab
        let filtered_torrents: Vec<_> = match self.top_tab {
            TopTab::All => torrents.to_vec(),
            TopTab::Completed => torrents
                .iter()
                .filter(|t| t.percent_done == Some(1.0))
                .cloned()
                .collect(),
            TopTab::Downloading => torrents
                .iter()
                .filter(|t| matches!(t.status, Some(TorrentStatus::Downloading)))
                .cloned()
                .collect(),
            TopTab::Seeding => torrents
                .iter()
                .filter(|t| matches!(t.status, Some(TorrentStatus::Seeding)))
                .cloned()
                .collect(),
        };

        // Top pane: torrents table
        let table = TorrentTable {
            torrents: &filtered_torrents,
        };
        table.render(frame, chunks[1], &mut self.table_state, &self.theme);

        // Bottom tabs
        let bottom_tab_titles = ["Details", "Peers", "Files"];
        render_tabs(
            &bottom_tab_titles,
            self.bottom_tab as usize,
            chunks[2],
            frame,
            self.focused_pane == Pane::Bottom,
            &self.theme,
        );

        // Bottom pane: details/peers/files
        match self.bottom_tab {
            BottomTab::Details => {
                let selected_torrent = self
                    .table_state
                    .selected()
                    .and_then(|n| filtered_torrents.get(n))
                    .cloned();
                let d = details::Details {
                    torrent: selected_torrent,
                };
                d.render(frame, chunks[3]);
            }
            BottomTab::Peers => {
                // TODO: Render peers table for selected torrent
                // Use self.peer_table_state for selection
            }
            BottomTab::Files => {
                // TODO: Render files table for selected torrent
                // Use self.file_table_state for selection
            }
        }
    }

    async fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key).await,
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    async fn on_key_event(&mut self, key: KeyEvent) {
        let torrents_len = self.torrents.lock().await.len();
        let key_event_str = keyevent_to_string(&key);
        let action = self.key_map.get(&key_event_str);
        if !action.is_some() {
            return;
        }

        match *action.unwrap() {
            // Quit commands
            Actions::Quit => self.quit(),

            // Pane focus
            Actions::FocusBottom => self.focused_pane = Pane::Bottom,
            Actions::FocusTop => self.focused_pane = Pane::Top,

            // Top pane navigation
            Actions::RowDown if self.focused_pane == Pane::Top => {
                let next = match self.table_state.selected() {
                    Some(i) if i + 1 < torrents_len => i + 1,
                    _ if torrents_len > 0 => 0,
                    _ => return,
                };
                self.table_state.select(Some(next));
            }
            Actions::RowUp if self.focused_pane == Pane::Top => {
                let prev = match self.table_state.selected() {
                    Some(0) if torrents_len > 0 => torrents_len - 1,
                    Some(i) => i - 1,
                    None if torrents_len > 0 => torrents_len - 1,
                    _ => return,
                };
                self.table_state.select(Some(prev));
            }
            // Top pane tab switching
            Actions::TabLeft if self.focused_pane == Pane::Top => {
                self.top_tab = match self.top_tab {
                    TopTab::All => TopTab::Seeding,
                    TopTab::Completed => TopTab::All,
                    TopTab::Downloading => TopTab::Completed,
                    TopTab::Seeding => TopTab::Downloading,
                };
            }
            Actions::TabRight if self.focused_pane == Pane::Top => {
                self.top_tab = match self.top_tab {
                    TopTab::All => TopTab::Completed,
                    TopTab::Completed => TopTab::Downloading,
                    TopTab::Downloading => TopTab::Seeding,
                    TopTab::Seeding => TopTab::All,
                };
            }

            // Bottom pane navigation (example for peers)
            Actions::RowDown
                if self.focused_pane == Pane::Bottom && self.bottom_tab == BottomTab::Peers =>
            {
                // TODO: logic for self.peer_table_state
            }
            Actions::RowUp
                if self.focused_pane == Pane::Bottom && self.bottom_tab == BottomTab::Peers =>
            {
                // TODO: logic for self.peer_table_state
            }
            // Bottom pane tab switching
            Actions::TabLeft if self.focused_pane == Pane::Bottom => {
                self.bottom_tab = match self.bottom_tab {
                    BottomTab::Details => BottomTab::Files,
                    BottomTab::Peers => BottomTab::Details,
                    BottomTab::Files => BottomTab::Peers,
                };
            }
            Actions::TabRight if self.focused_pane == Pane::Bottom => {
                self.bottom_tab = match self.bottom_tab {
                    BottomTab::Details => BottomTab::Peers,
                    BottomTab::Peers => BottomTab::Files,
                    BottomTab::Files => BottomTab::Details,
                };
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

fn render_tabs<'a, T: ToString>(
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
        base_style
    };
    let tabs = Tabs::new(titles)
        .select(selected)
        .block(Block::default().borders(Borders::ALL))
        .style(base_style)
        .highlight_style(highlight_style);
    frame.render_widget(tabs, area);
}
