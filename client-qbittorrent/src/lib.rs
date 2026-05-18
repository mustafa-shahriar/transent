use async_trait::async_trait;
use client_core::{
    ClientError, File, Peer, Priority, Status, Torrent, TorrentClient, TorrentDetails,
};
use std::sync::Arc;
use tokio::sync::Mutex;

// ─────────────────────────────────────────────
// ─── qBittorrent client ───────────────────────
// ─────────────────────────────────────────────
//
// API reference: https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)
//
// Authentication: cookie-based (SID cookie returned by /api/v2/auth/login).

pub struct QBittorrentClient {
    base_url: String,
    http: reqwest::Client,
    /// SID cookie value – populated after `login()`.
    sid: Arc<Mutex<String>>,
}

impl QBittorrentClient {
    /// Create a new client.  Call `login()` before issuing any other requests.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
            sid: Arc::new(Mutex::new(String::new())),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(), ClientError> {
        let url = format!("{}/api/v2/auth/login", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .header("Referer", &self.base_url)
            .form(&[("username", username), ("password", password)])
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;

        if resp.status() == 403 {
            return Err(ClientError::Backend(
                "IP banned due to too many failed login attempts".into(),
            ));
        }
        if !resp.status().is_success() {
            return Err(ClientError::Http(format!(
                "Login failed with HTTP {}",
                resp.status()
            )));
        }

        // Extract the SID cookie.
        let sid = resp
            .cookies()
            .find(|c| c.name() == "SID")
            .map(|c| c.value().to_owned())
            .ok_or_else(|| ClientError::Protocol("No SID cookie in login response".into()))?;

        *self.sid.lock().await = sid;
        Ok(())
    }

    // ── helpers ──────────────────────────────────────────────────────────

    fn api(&self, path: &str) -> String {
        format!(
            "{}/api/v2/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn sid_header(&self) -> String {
        format!("SID={}", self.sid.lock().await)
    }

    async fn get(&self, path: &str) -> Result<reqwest::Response, ClientError> {
        let resp = self
            .http
            .get(self.api(path))
            .header("Cookie", self.sid_header().await)
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ClientError::Http(format!("HTTP {}", resp.status())));
        }
        Ok(resp)
    }

    async fn post_form(
        &self,
        path: &str,
        form: &[(&str, &str)],
    ) -> Result<reqwest::Response, ClientError> {
        let resp = self
            .http
            .post(self.api(path))
            .header("Cookie", self.sid_header().await)
            .form(form)
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ClientError::Http(format!("HTTP {}", resp.status())));
        }
        Ok(resp)
    }

    fn qb_state(state: &str) -> Status {
        match state {
            "downloading" | "forcedDL" | "metaDL" => Status::Downloading,
            "uploading" | "forcedUP" | "stalledUP" => Status::Seeding,
            "pausedDL" | "pausedUP" | "stoppedDL" | "stoppedUP" => Status::Paused,
            "checkingDL" | "checkingUP" | "checkingResumeData" => Status::Checking,
            "queuedDL" | "queuedUP" => Status::Queued,
            _ => Status::Unknown,
        }
    }

    fn qb_priority(n: i64) -> Priority {
        match n {
            1 => Priority::Normal,
            6 => Priority::High,
            7 => Priority::Maximum,
            _ => Priority::Low, // 0 = do not download; treat as low for our model
        }
    }

    fn priority_to_qb(p: &Priority) -> &'static str {
        match p {
            Priority::Low => "0",
            Priority::Normal => "1",
            Priority::High => "6",
            Priority::Maximum => "7",
        }
    }
}

#[async_trait]
impl TorrentClient for QBittorrentClient {
    async fn get_torrents(&self) -> Result<Vec<Torrent>, ClientError> {
        let json: serde_json::Value = self
            .get("torrents/info")
            .await?
            .json()
            .await
            .map_err(|e| ClientError::Protocol(e.to_string()))?;

        let torrents = json
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|t| Torrent {
                info_hash: t["hash"].as_str().unwrap_or("").to_owned(),
                name: t["name"].as_str().unwrap_or("").to_owned(),
                down_speed: t["dlspeed"].as_u64().unwrap_or(0),
                up_speed: t["upspeed"].as_u64().unwrap_or(0),
                eta: t["eta"].as_u64().unwrap_or(0),
                progress: t["progress"].as_f64().unwrap_or(0.0),
                status: Self::qb_state(t["state"].as_str().unwrap_or("")),
            })
            .collect();

        Ok(torrents)
    }

    async fn get_torrent_details(&self, info_hash: &str) -> Result<TorrentDetails, ClientError> {
        // /torrents/properties gives us per-torrent stats.
        let props: serde_json::Value = self
            .get(&format!("torrents/properties?hash={}", info_hash))
            .await?
            .json()
            .await
            .map_err(|e| ClientError::Protocol(e.to_string()))?;

        // We also need the `state` field from the list endpoint.
        let list: serde_json::Value = self
            .get(&format!("torrents/info?hashes={}", info_hash))
            .await?
            .json()
            .await
            .map_err(|e| ClientError::Protocol(e.to_string()))?;

        let state = list
            .as_array()
            .and_then(|a| a.first())
            .and_then(|t| t["state"].as_str())
            .unwrap_or("");

        let eta_raw = props["eta"].as_i64().unwrap_or(-1);

        Ok(TorrentDetails {
            status: Self::qb_state(state),
            peers_count: props["peers"].as_u64().unwrap_or(0) as usize,
            total_downloaded: props["total_downloaded"].as_u64().unwrap_or(0) as usize,
            total_uploaded: props["total_uploaded"].as_u64().unwrap_or(0) as usize,
            up_speed: props["up_speed"].as_u64().unwrap_or(0) as usize,
            down_speed: props["dl_speed"].as_u64().unwrap_or(0) as usize,
            size: props["total_size"].as_u64().unwrap_or(0) as usize,
            eta: if eta_raw >= 0 {
                Some(eta_raw as usize)
            } else {
                None
            },
            seed_time: props["seeding_time"].as_u64().map(|v| v as usize),
        })
    }

    async fn get_torrent_peers(&self, info_hash: &str) -> Result<Vec<Peer>, ClientError> {
        // /sync/torrentPeers?hash=<hash>
        let json: serde_json::Value = self
            .get(&format!("sync/torrentPeers?hash={}", info_hash))
            .await?
            .json()
            .await
            .map_err(|e| ClientError::Protocol(e.to_string()))?;

        let peers_map = json["peers"].as_object().cloned().unwrap_or_default();

        let peers = peers_map
            .values()
            .map(|p| Peer {
                address: p["ip"].as_str().unwrap_or("").to_owned(),
                client: p["client"].as_str().unwrap_or("").to_owned(),
                up_speed: p["up_speed"].as_u64().unwrap_or(0) as usize,
                down_speed: p["dl_speed"].as_u64().unwrap_or(0) as usize,
            })
            .collect();

        Ok(peers)
    }

    async fn get_torrent_files(&self, info_hash: &str) -> Result<Vec<File>, ClientError> {
        let json: serde_json::Value = self
            .get(&format!("torrents/files?hash={}", info_hash))
            .await?
            .json()
            .await
            .map_err(|e| ClientError::Protocol(e.to_string()))?;

        let files = json
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|f| {
                let size = f["size"].as_u64().unwrap_or(1);
                let progress = f["progress"].as_f64().unwrap_or(0.0);
                // priority 0 in qBittorrent means "do not download"
                let priority_n = f["priority"].as_i64().unwrap_or(1);
                File {
                    is_selected: priority_n != 0,
                    name: f["name"].as_str().unwrap_or("").to_owned(),
                    size: size as usize,
                    progress,
                    priority: Self::qb_priority(priority_n),
                }
            })
            .collect();

        Ok(files)
    }

    async fn remove_torrent(&self, info_hash: &str, with_data: bool) -> Result<(), ClientError> {
        let delete_files = if with_data { "true" } else { "false" };
        self.post_form(
            "torrents/delete",
            &[("hashes", info_hash), ("deleteFiles", delete_files)],
        )
        .await?;
        Ok(())
    }

    async fn pause_torrent(&self, info_hash: &str) -> Result<(), ClientError> {
        self.post_form("torrents/pause", &[("hashes", info_hash)])
            .await?;
        Ok(())
    }

    async fn resume_torrent(&self, info_hash: &str) -> Result<(), ClientError> {
        self.post_form("torrents/resume", &[("hashes", info_hash)])
            .await?;
        Ok(())
    }

    async fn set_file_priority(
        &self,
        info_hash: &str,
        file_index: usize,
        priority: Priority,
    ) -> Result<(), ClientError> {
        let idx = file_index.to_string();
        let prio = Self::priority_to_qb(&priority);
        self.post_form(
            "torrents/filePrio",
            &[
                ("hash", info_hash),
                ("id", idx.as_str()),
                ("priority", prio),
            ],
        )
        .await?;
        Ok(())
    }

    async fn set_file_wanted_status(
        &self,
        info_hash: &str,
        file_index: usize,
        wanted: bool,
    ) -> Result<(), ClientError> {
        // priority 0 = do not download; 1 = normal
        let prio = if wanted { "1" } else { "0" };
        let idx = file_index.to_string();
        self.post_form(
            "torrents/filePrio",
            &[
                ("hash", info_hash),
                ("id", idx.as_str()),
                ("priority", prio),
            ],
        )
        .await?;
        Ok(())
    }
}
