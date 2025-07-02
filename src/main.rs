use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::TableState,
};
use transmission_rpc::{TransClient, types::Torrent};
mod components;
use components::torrent_table::TorrentTable;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use url::Url;

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
    let len = torrents.len();

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

    let app = App::new_arc(torrents_arc, len);

    let terminal = ratatui::init();
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
pub struct App {
    running: bool,
    torrents: Arc<Mutex<Vec<Torrent>>>,
    torrents_len: usize,
    pub table_state: TableState,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(torrents: Vec<Torrent>) -> Self {
        App {
            running: true,
            torrents: Arc::new(Mutex::new(torrents)),
            table_state: TableState::default(),
            torrents_len: 0,
        }
    }

    pub fn new_arc(torrents: Arc<Mutex<Vec<Torrent>>>, len: usize) -> Self {
        App {
            torrents_len: len,
            running: true,
            torrents,
            table_state: TableState::default(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            // Lock and clone the torrents for rendering
            let torrents = { self.torrents.lock().await.clone() };
            self.torrents_len = torrents.len();
            terminal.draw(|frame| self.render(frame, torrents))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, torrents: Vec<Torrent>) {
        let split = Layout::default()
            .margin(1)
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        let table = TorrentTable { torrents: torrents };
        table.render(frame, split[0], &mut self.table_state);
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(900))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        let torrents_len = self.torrents_len;

        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('j')) => {
                let next = match self.table_state.selected() {
                    Some(i) if i + 1 < torrents_len => i + 1,
                    _ => 0,
                };
                self.table_state.select(Some(next));
            }
            (_, KeyCode::Char('k')) => {
                let prev = match self.table_state.selected() {
                    Some(0) | None if torrents_len > 0 => torrents_len - 1,
                    Some(i) => i - 1,
                    None => 0,
                };
                self.table_state.select(Some(prev));
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}
