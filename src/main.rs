mod app;
mod config;
mod util;
mod widgets;

use crate::app::App;
use crate::util::get_client;
use crate::util::get_config;

use transmission_rpc::types::Torrent;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio::time::sleep;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tokio_main()
}

#[tokio::main]
async fn tokio_main() -> color_eyre::Result<()> {
    let config = get_config();
    let client = get_client(&config.rpc_config).unwrap();
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

    let app = App::new(client, torrents_arc, config.theme);

    let terminal = ratatui::init();
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}
