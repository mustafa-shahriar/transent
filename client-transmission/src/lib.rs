// ─────────────────────────────────────────────
// ─── Transmission client ─────────────────────
// ─────────────────────────────────────────────
//
// Uses the JSON-RPC 2.0 API introduced in Transmission 4.1.0
// (rpc_version_semver 6.0.0).  All strings are snake_case.
//
// Transport: HTTP POST to <base_url>/transmission/rpc
// CSRF:      On the first call the server returns HTTP 409 with the
//            correct X-Transmission-Session-Id header.  We retry once.

use async_trait::async_trait;
use client_core::{
    ClientError, File, Peer, Priority, Status, Torrent, TorrentClient, TorrentDetails,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TransmissionClient {
    base_url: String,
    http: reqwest::Client,
    /// The CSRF session token – refreshed automatically on 409.
    session_id: Arc<Mutex<String>>,
    /// Optional HTTP Basic Auth credentials.
    credentials: Option<(String, String)>,
}

impl TransmissionClient {
    /// Create a client without authentication.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
            session_id: Arc::new(Mutex::new(String::new())),
            credentials: None,
        }
    }

    /// Create a client with HTTP Basic Auth credentials.
    /// Transmission's auth is configured via `rpc_authentication_required`
    /// in its settings; credentials are sent on every request.
    pub fn with_auth(
        base_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
            session_id: Arc::new(Mutex::new(String::new())),
            credentials: Some((username.into(), password.into())),
        }
    }

    fn rpc_url(&self) -> String {
        format!("{}/transmission/rpc", self.base_url.trim_end_matches('/'))
    }

    /// Low-level call.  Handles the 409 / CSRF dance automatically.
    /// HTTP Basic Auth is attached on every request when credentials are set.
    async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ClientError> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        for attempt in 0..2u8 {
            let sid = self.session_id.lock().await.clone();
            let mut req = self
                .http
                .post(self.rpc_url())
                .header("X-Transmission-Session-Id", &sid)
                .json(&body);

            if let Some((user, pass)) = &self.credentials {
                req = req.basic_auth(user, Some(pass));
            }

            let resp = req
                .send()
                .await
                .map_err(|e| ClientError::Http(e.to_string()))?;

            if resp.status() == 409 {
                // Grab the new session token and retry once.
                if attempt == 1 {
                    return Err(ClientError::Protocol(
                        "Persistent 409 – could not refresh session id".into(),
                    ));
                }
                let new_sid = resp
                    .headers()
                    .get("X-Transmission-Session-Id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_owned();
                *self.session_id.lock().await = new_sid;
                continue;
            }

            if !resp.status().is_success() {
                return Err(ClientError::Http(format!("HTTP {}", resp.status())));
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| ClientError::Protocol(e.to_string()))?;

            if let Some(err) = json.get("error") {
                let msg = err
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                return Err(ClientError::JsonRpc(msg));
            }

            return Ok(json["result"].clone());
        }
        unreachable!()
    }

    // ── helpers ──────────────────────────────────────────────────────────

    /// torrent_get for a single torrent identified by its hash string.
    async fn torrent_get(
        &self,
        info_hash: &str,
        fields: &[&str],
    ) -> Result<serde_json::Value, ClientError> {
        let result = self
            .call(
                "torrent_get",
                serde_json::json!({
                    "ids":   [info_hash],
                    "fields": fields
                }),
            )
            .await?;

        result["torrents"]
            .as_array()
            .and_then(|a| a.first().cloned())
            .ok_or_else(|| ClientError::Protocol("torrent not found".into()))
    }

    fn tr_status(n: i64) -> Status {
        match n {
            0 => Status::Paused,
            1 | 3 => Status::Queued,
            2 => Status::Checking,
            4 => Status::Downloading,
            5 | 6 => Status::Seeding,
            _ => Status::Unknown,
        }
    }

    fn tr_priority(n: i64) -> Priority {
        match n {
            -1 => Priority::Low,
            1 => Priority::High,
            _ => Priority::Normal,
        }
    }

    fn priority_to_tr(p: &Priority) -> &'static str {
        match p {
            Priority::Low => "priority_low",
            Priority::Normal => "priority_normal",
            Priority::High | Priority::Maximum => "priority_high",
        }
    }
}

#[async_trait]
impl TorrentClient for TransmissionClient {
    async fn get_torrents(&self) -> Result<Vec<Torrent>, ClientError> {
        let result = self
            .call(
                "torrent_get",
                serde_json::json!({
                    "fields": [
                        "hash_string", "name",
                        "rate_download", "rate_upload",
                        "eta", "percent_done", "status"
                    ]
                }),
            )
            .await?;

        let torrents = result["torrents"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|t| Torrent {
                info_hash: t["hash_string"].as_str().unwrap_or("").to_owned(),
                name: t["name"].as_str().unwrap_or("").to_owned(),
                down_speed: t["rate_download"].as_u64().unwrap_or(0),
                up_speed: t["rate_upload"].as_u64().unwrap_or(0),
                eta: t["eta"].as_u64().unwrap_or(0),
                progress: t["percent_done"].as_f64().unwrap_or(0.0),
                status: Self::tr_status(t["status"].as_i64().unwrap_or(-1)),
            })
            .collect();

        Ok(torrents)
    }

    async fn get_torrent_details(&self, info_hash: &str) -> Result<TorrentDetails, ClientError> {
        let t = self
            .torrent_get(
                info_hash,
                &[
                    "status",
                    "peers_connected",
                    "downloaded_ever",
                    "uploaded_ever",
                    "rate_upload",
                    "rate_download",
                    "total_size",
                    "eta",
                    "seconds_seeding",
                ],
            )
            .await?;

        Ok(TorrentDetails {
            status: Self::tr_status(t["status"].as_i64().unwrap_or(-1)),
            peers_count: t["peers_connected"].as_u64().unwrap_or(0) as usize,
            total_downloaded: t["downloaded_ever"].as_u64().unwrap_or(0) as usize,
            total_uploaded: t["uploaded_ever"].as_u64().unwrap_or(0) as usize,
            up_speed: t["rate_upload"].as_u64().unwrap_or(0) as usize,
            down_speed: t["rate_download"].as_u64().unwrap_or(0) as usize,
            size: t["total_size"].as_u64().unwrap_or(0) as usize,
            eta: t["eta"].as_i64().filter(|&v| v >= 0).map(|v| v as usize),
            seed_time: t["seconds_seeding"].as_u64().map(|v| v as usize),
        })
    }

    async fn get_torrent_peers(&self, info_hash: &str) -> Result<Vec<Peer>, ClientError> {
        let t = self.torrent_get(info_hash, &["peers"]).await?;

        let peers = t["peers"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|p| Peer {
                address: p["address"].as_str().unwrap_or("").to_owned(),
                client: p["client_name"].as_str().unwrap_or("").to_owned(),
                up_speed: p["rate_to_peer"].as_u64().unwrap_or(0) as usize,
                down_speed: p["rate_to_client"].as_u64().unwrap_or(0) as usize,
            })
            .collect();

        Ok(peers)
    }

    async fn get_torrent_files(&self, info_hash: &str) -> Result<Vec<File>, ClientError> {
        let t = self
            .torrent_get(info_hash, &["files", "file_stats"])
            .await?;

        let files = t["files"].as_array().cloned().unwrap_or_default();
        let stats = t["file_stats"].as_array().cloned().unwrap_or_default();

        let result = files
            .iter()
            .zip(stats.iter())
            .map(|(f, s)| {
                let length = f["length"].as_u64().unwrap_or(1);
                let completed = f["bytes_completed"].as_u64().unwrap_or(0);
                File {
                    is_selected: s["wanted"].as_bool().unwrap_or(false),
                    name: f["name"].as_str().unwrap_or("").to_owned(),
                    size: length as usize,
                    progress: completed as f64 / length as f64,
                    priority: Self::tr_priority(s["priority"].as_i64().unwrap_or(0)),
                }
            })
            .collect();

        Ok(result)
    }

    async fn remove_torrent(&self, info_hash: &str, with_data: bool) -> Result<(), ClientError> {
        self.call(
            "torrent_remove",
            serde_json::json!({
                "ids": [info_hash],
                "delete_local_data": with_data
            }),
        )
        .await?;
        Ok(())
    }

    async fn pause_torrent(&self, info_hash: &str) -> Result<(), ClientError> {
        self.call("torrent_stop", serde_json::json!({ "ids": [info_hash] }))
            .await?;
        Ok(())
    }

    async fn resume_torrent(&self, info_hash: &str) -> Result<(), ClientError> {
        self.call("torrent_start", serde_json::json!({ "ids": [info_hash] }))
            .await?;
        Ok(())
    }

    async fn set_file_priority(
        &self,
        info_hash: &str,
        file_index: usize,
        priority: Priority,
    ) -> Result<(), ClientError> {
        let key = Self::priority_to_tr(&priority);
        self.call(
            "torrent_set",
            serde_json::json!({
                "ids": [info_hash],
                key: [file_index]
            }),
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
        let key = if wanted {
            "files_wanted"
        } else {
            "files_unwanted"
        };
        self.call(
            "torrent_set",
            serde_json::json!({
                "ids": [info_hash],
                key: [file_index]
            }),
        )
        .await?;
        Ok(())
    }
}
