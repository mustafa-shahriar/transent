use color_eyre::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, TableState},
};
use transmission_rpc::{
    TransClient,
    types::{Id, Torrent, TorrentAction, TorrentAddArgs, TorrentStatus},
};
mod components;
use components::torrent_table::TorrentTable;
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use url::Url;

use crate::{
    components::{
        delete_confirmation::Popup, details, file_picker::FilePicker,
        peers_table::render_peers_table, tabs::render_tabs, util::get_entries,
    },
    key_config::load_keymap,
};
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
    let torrent_len = torrents.len();

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

    let app = App::new(torrents_arc, client, torrent_len);

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
    selected_torrent_id: Option<String>,
    delete_confirmation: bool,
    with_data: bool,
    client: Arc<Mutex<transmission_rpc::TransClient>>,
    pub table_state: TableState,
    pub peer_table_state: TableState,
    pub file_table_state: TableState,
    focused_pane: Pane,
    top_tab: TopTab,
    bottom_tab: BottomTab,
    pub theme: Theme,
    pub key_map: HashMap<String, Actions>,
    visible_torrents_len: usize,
    show_file_picker: bool,
    file_picker: FilePicker,
    file_picker_state: TableState,
}

impl App {
    pub fn new(
        torrents: Arc<Mutex<Vec<Torrent>>>,
        client: Arc<Mutex<TransClient>>,
        torrents_len: usize,
    ) -> Self {
        let key_map = load_keymap("key_config.toml");
        App {
            running: true,
            torrents,
            delete_confirmation: false,
            with_data: false,
            selected_torrent_id: None,
            client,
            table_state: TableState::default(),
            peer_table_state: TableState::default(),
            file_table_state: TableState::default(),
            focused_pane: Pane::Top,
            top_tab: TopTab::All,
            bottom_tab: BottomTab::Details,
            theme: load_theme(),
            key_map: key_map,
            visible_torrents_len: torrents_len,
            show_file_picker: false,
            file_picker: FilePicker::new("~/".to_string()),
            file_picker_state: TableState::default(),
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
            .margin(1)
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
        self.visible_torrents_len = filtered_torrents.len();

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
        let selected_torrent = self
            .table_state
            .selected()
            .and_then(|n| filtered_torrents.get(n))
            .cloned();

        let mut name = "".to_string();
        match selected_torrent.as_ref() {
            Some(torrent) => {
                self.selected_torrent_id = torrent.hash_string.clone();
                name = selected_torrent.as_ref().unwrap().name.clone().unwrap();
            }
            None => self.selected_torrent_id = None,
        }

        match self.bottom_tab {
            BottomTab::Details => {
                let d = details::Details {
                    torrent: selected_torrent,
                };
                d.render(frame, chunks[3]);
            }
            BottomTab::Peers => match selected_torrent {
                Some(torrent) => render_peers_table(
                    torrent,
                    frame,
                    chunks[3],
                    &mut self.peer_table_state,
                    &self.theme,
                ),
                None => {
                    let p = Paragraph::new("Select A torrent to view Peers List");
                    frame.render_widget(p, chunks[3]);
                }
            },
            BottomTab::Files => {
                // TODO: Render files table for selected torrent
                // Use self.file_table_state for selection
            }
        }
        if self.delete_confirmation {
            let popup_area = Rect {
                x: frame.area().width / 4,
                y: frame.area().height / 3,
                width: frame.area().width / 2,
                height: frame.area().height / 2,
            };
            let content = format!("{}\n\n\n\n[Y]es    [N]o", name);
            let title = if self.with_data {
                "Delete icluding data"
            } else {
                "Delete excluding data"
            };
            let popup = Popup::default().content(content).title(title);
            frame.render_widget(popup, popup_area);
        }

        if self.show_file_picker {
            let popup_area = Rect {
                x: frame.area().width / 4,
                y: frame.area().height / 3,
                width: frame.area().width / 2,
                height: frame.area().height / 2,
            };
            self.file_picker
                .render(frame, popup_area, &mut self.file_picker_state, &self.theme);
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
        let torrents_len = self.visible_torrents_len;
        let key_event_str = keyevent_to_string(&key);

        if self.delete_confirmation {
            match key_event_str.as_ref() {
                "y" | "Y" => {
                    let ids = vec![Id::Hash(self.selected_torrent_id.clone().unwrap())];
                    let mut client = self.client.lock().await;
                    let _ = client.torrent_remove(ids, self.with_data).await;
                    self.delete_confirmation = false;
                }
                "n" | "N" => {
                    self.delete_confirmation = false;
                }
                _ => {}
            }
            return;
        }

        let action = self.key_map.get(&key_event_str);
        if action.is_none() {
            return;
        }

        if self.show_file_picker {
            match *action.unwrap() {
                Actions::Quit => {
                    self.show_file_picker = false;
                }
                Actions::RowDown => {
                    self.file_picker_state.select_next();
                }
                Actions::RowUp => {
                    self.file_picker_state.select_previous();
                }
                Actions::TabRight => match self.file_picker_state.selected() {
                    Some(n) => {
                        self.file_picker.previos_indexes.push(n);
                        let selected_path = self.file_picker.entries[n]
                            .path()
                            .canonicalize()
                            .unwrap()
                            .display()
                            .to_string();
                        self.file_picker.path = selected_path;
                        if self.file_picker.path.ends_with(".torrent") {
                            let mut t = TorrentAddArgs::default();
                            t.files_unwanted = None;
                            t.filename = Some(self.file_picker.path.clone());
                            let r = self.client.lock().await.torrent_add(t).await;
                            match r {
                                Ok(_) => self.show_file_picker = false,
                                Err(_) => {}
                            }
                            return;
                        };
                        let entries = get_entries(self.file_picker.path.clone());
                        match entries.len() {
                            0 => self.file_picker_state.select(None),
                            _ => self.file_picker_state.select(Some(1)),
                        }
                        self.file_picker.entries = entries;
                    }
                    None => {}
                },
                Actions::TabLeft => {
                    let path = Path::new(&self.file_picker.path);
                    match path.parent() {
                        Some(parent) => {
                            self.file_picker.path = parent.display().to_string();
                            self.file_picker.entries = get_entries(self.file_picker.path.clone());
                            self.file_picker_state
                                .select(self.file_picker.previos_indexes.pop());
                        }
                        None => {}
                    }
                }
                _ => {}
            }
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
                self.peer_table_state.select_next();
            }
            Actions::RowUp
                if self.focused_pane == Pane::Bottom && self.bottom_tab == BottomTab::Peers =>
            {
                self.peer_table_state.select_previous();
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
            Actions::Resume if self.selected_torrent_id.is_some() => {
                let ids = vec![Id::Hash(self.selected_torrent_id.clone().unwrap())];
                let mut client = self.client.lock().await;
                let _ = client.torrent_action(TorrentAction::Start, ids).await;
            }
            Actions::Pause if self.selected_torrent_id.is_some() => {
                let ids = vec![Id::Hash(self.selected_torrent_id.clone().unwrap())];
                let mut client = self.client.lock().await;
                let _ = client.torrent_action(TorrentAction::Stop, ids).await;
            }
            Actions::Delete if self.selected_torrent_id.is_some() => {
                self.delete_confirmation = true;
                self.with_data = false;
            }
            Actions::DeleteWithData if self.selected_torrent_id.is_some() => {
                self.delete_confirmation = true;
                self.with_data = true;
            }
            Actions::AddTorrent => {
                self.show_file_picker = true;
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}
