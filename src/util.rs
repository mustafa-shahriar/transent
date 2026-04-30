use home::home_dir;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use std::cmp::Ordering;
use std::fs;
use std::fs::DirEntry;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use transmission_rpc::TransClient;
use transmission_rpc::types::TorrentStatus;
use url::Url;

use crate::config::Config;
use crate::config::RpcConfig;

pub fn get_client(rpc_config: &RpcConfig) -> color_eyre::Result<Arc<Mutex<TransClient>>> {
    let mut url = Url::parse(&rpc_config.url)?;

    url.set_username(&rpc_config.username)
        .map_err(|_| color_eyre::eyre::eyre!("invalid username"))?;

    url.set_password(Some(&rpc_config.password))
        .map_err(|_| color_eyre::eyre::eyre!("invalid password"))?;

    let client = Arc::new(Mutex::new(TransClient::new(url)));
    Ok(client)
}

pub fn get_config() -> Config {
    let path = get_conf_dir().join("config.toml");
    let path = path.to_str().unwrap();
    let content = fs::read_to_string(path).expect("config.toml not found");
    toml::from_str(&content).expect("Invalid config.toml")
}

pub fn expand_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Some(str_path) = p.to_str()
        && let Some(home) = home_dir()
        && let Some(stripped) = str_path.strip_prefix("~/")
    {
        return home.join(stripped);
    }
    p.to_path_buf()
}

pub fn get_entries(path: String, show_hidden: bool) -> Vec<DirEntry> {
    let real_path = expand_path(path);

    let mut entries: Vec<DirEntry> = match read_dir(&real_path) {
        Ok(entries) => entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();

                    let is_hidden = e
                        .file_name()
                        .to_str()
                        .map(|name| name.starts_with('.'))
                        .unwrap_or(false);

                    if !show_hidden && is_hidden {
                        return None;
                    }

                    if path.is_dir()
                        || (path.is_file()
                            && path
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext.eq_ignore_ascii_case("torrent"))
                                .unwrap_or(false))
                    {
                        Some(e)
                    } else {
                        None
                    }
                })
            })
            .collect(),
        Err(_) => return vec![],
    };

    entries.sort_by(|a, b| {
        let a_path = a.path();
        let b_path = b.path();

        match (a_path.is_dir(), b_path.is_dir()) {
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            _ => {}
        }

        let a_name = a.file_name().to_string_lossy().to_lowercase();
        let b_name = b.file_name().to_string_lossy().to_lowercase();

        a_name.cmp(&b_name)
    });

    entries
}

pub fn fuzzy_match(text: &str, query: &str) -> bool {
    let mut query_chars = query.chars();

    let mut current = match query_chars.next() {
        Some(c) => c,
        None => return true,
    };

    for c in text.chars() {
        if c == current {
            match query_chars.next() {
                Some(next) => current = next,
                None => return true,
            }
        }
    }

    false
}

/// Calculate a match score for ranking fuzzy matches.
/// Higher scores indicate better matches.
pub fn calculate_match_score(text: &str, query: &str) -> usize {
    let mut score = 0;
    let mut query_chars = query.chars().peekable();
    let mut last_match_pos = 0;

    for (pos, c) in text.chars().enumerate() {
        if let Some(&query_char) = query_chars.peek()
            && c == query_char
        {
            query_chars.next();

            // Bonus for consecutive matches
            if pos == last_match_pos + 1 {
                score += 10; // Bonus for consecutive character match
            } else {
                score += 5; // Base score for a match
            }

            // Bonus for matches at the start of the string
            if pos == 0 {
                score += 20;
            }

            last_match_pos = pos;
        }
    }

    // Bonus for completing the entire query
    if query_chars.peek().is_none() {
        score += 50;
    }

    score
}

pub fn readabl_eta(eta: i64) -> String {
    if eta < 0 {
        return "∞".to_string();
    }

    let duration = Duration::from_secs(eta as u64);
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours >= 1 {
        format!("{}h{}m", hours, minutes)
    } else if minutes >= 1 {
        format!("{}m{}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn round_to_2_decimals(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub fn readable_size(size: u64) -> String {
    let size = size as f64;
    let kb = 1024.0;
    let mb = kb * 1024.0;
    let gb = mb * 1024.0;
    if size >= gb {
        return round_to_2_decimals(size / gb).to_string() + " GB";
    } else if size >= mb {
        return round_to_2_decimals(size / mb).to_string() + " MB";
    }
    round_to_2_decimals(size).to_string() + " KB"
}

pub fn readable_time(sec: i64) -> String {
    let duration = Duration::from_secs(sec as u64);
    let secs = duration.as_secs();
    let seconds = secs % 60;
    let minutes = (secs % 3600) / 60;
    let hours = secs / 3600;
    let days = secs / 86400;

    if days >= 1 {
        format!("{}d{}h{}m", days, hours, minutes)
    } else if hours >= 1 {
        format!("{}h{}m{}s", hours, minutes, seconds)
    } else if minutes >= 1 {
        format!("{}m{}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn status_to_string(status: TorrentStatus) -> String {
    let s = match status {
        TorrentStatus::Stopped => "Stopped",
        TorrentStatus::QueuedToVerify => "QueuedToVerify",
        TorrentStatus::Verifying => "Verifying",
        TorrentStatus::QueuedToDownload => "QueuedToDownload",
        TorrentStatus::Downloading => "Downloading",
        TorrentStatus::QueuedToSeed => "QueuedToSeed",
        TorrentStatus::Seeding => "Seeding",
    };

    s.to_string()
}

pub fn readble_speed(byte: i64) -> String {
    let byte = byte as f64;
    let kilo_byte = 1024.0;
    let mega_byte = kilo_byte * 1024.0;
    let giga_byte = mega_byte * 1024.0;

    if byte >= giga_byte {
        return round_to_2_decimals(byte / giga_byte).to_string() + " GB/s";
    } else if byte >= mega_byte {
        return round_to_2_decimals(byte / mega_byte).to_string() + " MB/s";
    } else if byte >= kilo_byte {
        return round_to_2_decimals(byte / kilo_byte).to_string() + " KB/s";
    }

    round_to_2_decimals(byte).to_string() + " B/s"
}

pub fn get_conf_dir() -> PathBuf {
    let config_dir = dirs::config_dir().expect("Could not find config directory");
    config_dir.join("transent")
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn icon_for(entry: &DirEntry) -> &'static str {
    let path = entry.path();

    if path.is_dir() {
        return "📁";
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "torrent" => "🌊", // Torrent
            _ => "📄",         // Default for other files
        }
    } else {
        "📄"
    }
}
