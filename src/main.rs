mod app;
mod theme;
mod util;
mod widgets;

use crate::app::App;
use crate::theme::Theme;
use crate::util::get_conf_dir;

use transmission_rpc::TransClient;
use transmission_rpc::types::Torrent;

use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio::time::sleep;
use url::Url;

fn get_theme() -> Theme {
    let path = get_conf_dir().join("theme.toml");
    let path = path.to_str().unwrap();
    let content = fs::read_to_string(path).expect("theme.toml not found");
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
    let torrents: Vec<Torrent> = vec![];
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

    let app = App::new(client, torrents_arc, get_theme());

    let terminal = ratatui::init();
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}
