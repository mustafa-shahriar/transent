use crate::theme::Theme;
use crate::widgets::custome_tab::CustomeTabs;
use crate::widgets::delete_popup::DeletePopup;
use crate::widgets::file_picker::FilePicker;
use crate::widgets::files_table::FilesTable;
use crate::widgets::peers_table::PeersTable;
use crate::widgets::torrent_actions::TorrentActions;
use crate::widgets::torrent_details::Details;
use crate::widgets::torrent_table::TorrentTable;

use color_eyre::Result;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use dirs::home_dir;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::layout::Constraint::Length;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::widgets::ScrollbarState;
use ratatui::widgets::TableState;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use transmission_rpc::types::Id;
use transmission_rpc::types::Torrent;
use transmission_rpc::types::TorrentAction;
use transmission_rpc::types::TorrentStatus;

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

impl fmt::Display for TopTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TopTab::All => "All",
            TopTab::Completed => "Completed",
            TopTab::Downloading => "Downloading",
            TopTab::Seeding => "Seeding",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TopTab {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "All" => Ok(TopTab::All),
            "Completed" => Ok(TopTab::Completed),
            "Downloading" => Ok(TopTab::Downloading),
            "Seeding" => Ok(TopTab::Seeding),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum BottomTab {
    Details,
    Peers,
    Files,
}

impl fmt::Display for BottomTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BottomTab::Details => "Details",
            BottomTab::Peers => "Peers",
            BottomTab::Files => "Files",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for BottomTab {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Details" => Ok(BottomTab::Details),
            "Peers" => Ok(BottomTab::Peers),
            "Files" => Ok(BottomTab::Files),
            _ => Err(()),
        }
    }
}

pub enum PopUp {
    TorrentAction(TorrentActions),
    DeleteConfirmation(DeletePopup),
    FilePicker,
}

pub struct BottomPane {
    pub details_block: Details,
    pub peers_table: PeersTable,
    pub files_table: FilesTable,
}

pub struct App {
    running: bool,
    client: Arc<Mutex<transmission_rpc::TransClient>>,
    all_torrents: Arc<Mutex<Vec<Torrent>>>,
    top_tab: CustomeTabs,
    top_table: TorrentTable,
    bottom_tab: CustomeTabs,
    bottom_pane: BottomPane,
    active_pane: Pane,
    popup: Option<PopUp>,
    file_picker: FilePicker,
    theme: Theme,
}

impl App {
    pub fn new(
        client: Arc<Mutex<transmission_rpc::TransClient>>,
        all_torrents: Arc<Mutex<Vec<Torrent>>>,
        theme: Theme,
    ) -> Self {
        App {
            client,
            all_torrents,
            theme,
            top_tab: CustomeTabs::new(
                vec![
                    TopTab::All.to_string(),
                    TopTab::Completed.to_string(),
                    TopTab::Downloading.to_string(),
                    TopTab::Seeding.to_string(),
                ],
                true,
            ),
            bottom_tab: CustomeTabs::new(
                vec![
                    BottomTab::Details.to_string(),
                    BottomTab::Peers.to_string(),
                    BottomTab::Files.to_string(),
                ],
                false,
            ),
            top_table: TorrentTable {
                torrents: vec![],
                state: TableState::default(),
                scrollbar_state: ScrollbarState::default(),
            },
            bottom_pane: BottomPane {
                details_block: Details::new(),
                files_table: FilesTable::new(),
                peers_table: PeersTable::new(),
            },
            file_picker: FilePicker::new(home_dir().unwrap().to_str().unwrap().to_string(), false),
            active_pane: Pane::Top,
            popup: None,
            running: true,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            self.filter_torrents().await;
            self.set_data_bottom_pane().await;
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![Length(3), Percentage(47), Length(3), Percentage(47)])
            .split(frame.area());

        self.top_tab.render(frame, chunks[0], &self.theme);
        self.top_table.render(frame, chunks[1], &self.theme);
        self.bottom_tab.render(frame, chunks[2], &self.theme);

        match self.bottom_tab.selected_tab().parse().unwrap() {
            BottomTab::Details => {
                self.bottom_pane
                    .details_block
                    .render(frame, chunks[3], &self.theme)
            }
            BottomTab::Peers => {
                self.bottom_pane
                    .peers_table
                    .render(frame, chunks[3], &self.theme);
            }
            BottomTab::Files => {
                self.bottom_pane
                    .files_table
                    .render(frame, chunks[3], &self.theme);
            }
        }

        if self.popup.is_none() {
            return;
        }
        match self.popup.as_mut().unwrap() {
            PopUp::TorrentAction(ta) => ta.render(frame, &self.theme),
            PopUp::DeleteConfirmation(dc) => dc.render(frame, &self.theme),
            PopUp::FilePicker => self.file_picker.render(frame, &self.theme),
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
        if self.popup.is_some() {
            match self.popup.as_ref().unwrap() {
                PopUp::DeleteConfirmation(_) => self.handle_popup_delete(key).await,
                PopUp::TorrentAction(_) => self.handle_actions_menu(key).await,
                PopUp::FilePicker => self.handle_filepicker(key).await,
            }
            return;
        }
        if key.code.to_string() == "q" {
            self.running = false;
            return;
        }
        if key.code.to_string() == "a" {
            self.popup = Some(PopUp::FilePicker);
            return;
        }
        match self.active_pane {
            Pane::Top => self.handle_top_pane(key),
            Pane::Bottom => self.handle_bottom_pane(key),
        }
    }

    async fn handle_popup_delete(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let PopUp::DeleteConfirmation(p) = self.popup.as_ref().unwrap() {
                    self.delete_torrent(p.id.clone(), p.with_data).await;
                    self.popup = None;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('q') => self.popup = None,
            _ => {}
        }
    }

    async fn handle_actions_menu(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Char('q')) {
            self.popup = None;
            return;
        }

        let action = if let Some(PopUp::TorrentAction(a)) = self.popup.as_mut() {
            match key.code {
                KeyCode::Char('j') => {
                    a.select_next();
                    return;
                }
                KeyCode::Char('k') => {
                    a.select_prev();
                    return;
                }
                KeyCode::Enter => match a.get_selected() {
                    Some(s) => Some((s, a.id.clone())),
                    None => None,
                },
                _ => None,
            }
        } else {
            None
        };

        if let Some((selected, id)) = action {
            match selected {
                "Pause" => self.pause(id).await,
                "Resume" => self.resume(id).await,
                "Delete" => self.delete_torrent(id, false).await,
                "Delete including Data" => self.delete_torrent(id, true).await,
                _ => {}
            }
            self.popup = None;
        }
    }

    async fn handle_filepicker(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('q') {
            self.popup = None;
            return;
        }

        if self.file_picker.handler(key, &self.client).await {
            self.popup = None;
        }
    }

    fn handle_top_pane(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('j'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.active_pane = Pane::Bottom;
                self.top_tab.is_focused = false;
                self.bottom_tab.is_focused = true;
            }
            (KeyCode::Char('K'), _) => {
                let index = self.top_table.state.selected();
                if index.is_none() {
                    return;
                }
                let t = self.top_table.torrents.get(index.unwrap()).unwrap();
                let id = t.id().unwrap();
                let name = t.name.clone().unwrap();
                let popup = PopUp::TorrentAction(TorrentActions::new(id, name));
                self.popup = Some(popup);
            }
            (KeyCode::Char('D'), _) => {
                let index = self.top_table.state.selected();
                if index.is_none() {
                    return;
                }
                let t = self.top_table.torrents.get(index.unwrap()).unwrap();
                let id = t.id().unwrap();
                let name = t.name.clone().unwrap();
                let popup = PopUp::DeleteConfirmation(DeletePopup::new(id, name, true));
                self.popup = Some(popup);
            }
            (KeyCode::Char('d'), _) => {
                let index = self.top_table.state.selected();
                if index.is_none() {
                    return;
                }
                let t = self.top_table.torrents.get(index.unwrap()).unwrap();
                let id = t.id().unwrap();
                let name = t.name.clone().unwrap();
                let popup = PopUp::DeleteConfirmation(DeletePopup::new(id, name, false));
                self.popup = Some(popup);
            }
            (KeyCode::Char('j'), _) => {
                self.top_table.select_next();
            }
            (KeyCode::Char('k'), _) => {
                self.top_table.select_prev();
            }
            (KeyCode::Char('l'), _) => {
                self.top_tab.select_next();
            }
            (KeyCode::Char('h'), _) => {
                self.top_tab.select_prev();
            }
            _ => {}
        }
    }

    fn handle_bottom_pane(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('k'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.active_pane = Pane::Top;
                self.bottom_tab.is_focused = false;
                self.top_tab.is_focused = true;
            }
            (KeyCode::Char('l'), _) => {
                self.bottom_tab.select_next();
            }
            (KeyCode::Char('h'), _) => {
                self.bottom_tab.select_prev();
            }
            _ => {}
        }
    }

    async fn filter_torrents(&mut self) {
        let all_torrents = self.all_torrents.lock().await;
        let filtered_torrents: Vec<_> = match self.top_tab.selected_tab().parse().unwrap() {
            TopTab::All => all_torrents.clone(),
            TopTab::Completed => all_torrents
                .iter()
                .filter(|t| t.percent_done == Some(1.0))
                .cloned()
                .collect(),
            TopTab::Downloading => all_torrents
                .iter()
                .filter(|t| matches!(t.status, Some(TorrentStatus::Downloading)))
                .cloned()
                .collect(),
            TopTab::Seeding => all_torrents
                .iter()
                .filter(|t| matches!(t.status, Some(TorrentStatus::Seeding)))
                .cloned()
                .collect(),
        };
        self.top_table.torrents = filtered_torrents;
    }

    async fn set_data_bottom_pane(&mut self) {
        if self.top_table.state.selected().is_none() {
            return;
        };

        let selected_index = self.top_table.state.selected().unwrap();
        if selected_index >= self.top_table.torrents.len() {
            return;
        }

        let selected_torrent = self.top_table.torrents.get(selected_index).unwrap();

        match self.bottom_tab.selected_tab().parse().unwrap() {
            BottomTab::Files => {
                let files = selected_torrent.files.clone().unwrap();
                self.bottom_pane.files_table.files = files;
            }
            BottomTab::Peers => {
                let peers = selected_torrent.peers.clone().unwrap();
                self.bottom_pane.peers_table.peers = peers;
            }
            BottomTab::Details => {
                self.bottom_pane.details_block.torrent = Some(selected_torrent.clone());
            }
        }
    }

    async fn resume(&mut self, id: Id) {
        let mut client = self.client.lock().await;
        let _ = client.torrent_action(TorrentAction::Start, vec![id]).await;
    }
    async fn pause(&mut self, id: Id) {
        let mut client = self.client.lock().await;
        let _ = client.torrent_action(TorrentAction::Stop, vec![id]).await;
    }

    async fn delete_torrent(&mut self, id: Id, with_data: bool) {
        let mut client = self.client.lock().await;
        let _ = client.torrent_remove(vec![id], with_data).await;
    }
}
