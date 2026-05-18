use async_trait::async_trait;
use thiserror::Error;

// ─────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Not supported by this client: {0}")]
    Unsupported(&'static str),
}

// ─────────────────────────────────────────────
// Shared domain types
// ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    Low,
    Normal,
    High,
    Maximum,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Downloading,
    Seeding,
    Paused,
    Checking,
    Queued,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Torrent {
    pub info_hash: String,
    pub name: String,
    pub down_speed: u64,
    pub up_speed: u64,
    pub eta: u64,
    pub progress: f64,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct TorrentDetails {
    pub status: Status,
    pub peers_count: usize,
    pub total_downloaded: usize,
    pub total_uploaded: usize,
    pub up_speed: usize,
    pub down_speed: usize,
    pub size: usize,
    pub eta: Option<usize>,
    pub seed_time: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub address: String,
    pub client: String,
    pub up_speed: usize,
    pub down_speed: usize,
}

#[derive(Debug, Clone)]
pub struct File {
    pub is_selected: bool,
    pub name: String,
    pub size: usize,
    pub progress: f64,
    pub priority: Priority,
}

// ─────────────────────────────────────────────
// Trait  (note: &self receivers so the trait is
// object-safe, and methods return owned futures)
// ─────────────────────────────────────────────

#[async_trait]
pub trait TorrentClient: Send + Sync {
    async fn get_torrents(&self) -> Result<Vec<Torrent>, ClientError>;
    async fn get_torrent_details(&self, info_hash: &str) -> Result<TorrentDetails, ClientError>;
    async fn get_torrent_peers(&self, info_hash: &str) -> Result<Vec<Peer>, ClientError>;
    async fn get_torrent_files(&self, info_hash: &str) -> Result<Vec<File>, ClientError>;
    async fn remove_torrent(&self, info_hash: &str, with_data: bool) -> Result<(), ClientError>;
    async fn pause_torrent(&self, info_hash: &str) -> Result<(), ClientError>;
    async fn resume_torrent(&self, info_hash: &str) -> Result<(), ClientError>;
    async fn set_file_priority(
        &self,
        info_hash: &str,
        file_index: usize,
        priority: Priority,
    ) -> Result<(), ClientError>;
    async fn set_file_wanted_status(
        &self,
        info_hash: &str,
        file_index: usize,
        wanted: bool,
    ) -> Result<(), ClientError>;
}
