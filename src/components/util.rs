use home::home_dir;
use std::{
    cmp::Ordering,
    fs::{DirEntry, read_dir},
    path::{Path, PathBuf},
    time::Duration,
};
use transmission_rpc::types::TorrentStatus;

pub fn expand_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Some(str_path) = p.to_str() {
        if let Some(home) = home_dir() {
            if str_path.starts_with("~/") {
                return home.join(&str_path[2..]);
            }
        }
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

pub fn readable_size(size: i64) -> String {
    let size = size as f64;
    let kb = 1024.0;
    let mb = kb * 1024.0;
    let gb = mb * 1024.0;
    if size >= gb {
        return round_to_2_decimals(size / gb).to_string() + " GB";
    } else if size >= mb {
        return round_to_2_decimals(size / mb).to_string() + " MB";
    }
    return round_to_2_decimals(size).to_string() + " KB";
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

    return s.to_string();
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

    return round_to_2_decimals(byte).to_string() + " B/s";
}

pub fn get_conf_dir() -> PathBuf {
    let config_dir = dirs::config_dir().expect("Could not find config directory");
    config_dir.join("transent")
}
