use crate::clients::errors::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Struct to store/read in storage
pub trait ReadRequest {}
pub trait WriteRequest {}

pub trait ReadResult {}
pub trait WriteResult {}

/// Domain model representing a Last.fm session key, implementing the Document trait for storage operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct LastFMSessionKey {
    pub key: String,
}
impl LastFMSessionKey {
    pub fn get_key(&self) -> String {
        self.key.clone()
    }
}

pub struct KeyReadRequest {}
pub struct KeyWriteResult {}

pub enum KeyReadResult {
    Found(LastFMSessionKey),
    NotFound,
}

impl ReadRequest for KeyReadRequest {}
impl ReadResult for KeyReadResult {}
impl WriteRequest for LastFMSessionKey {}
impl WriteResult for KeyWriteResult {}

/// Domain model representing a Last.fm session key, implementing the Document trait for storage operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedTracks {
    track_ids: Vec<String>,
}

#[async_trait]
pub trait Storage: Send + Sync {
    type ReadReq;
    type ReadRes;
    type WriteReq;
    type WriteRes;

    /// Stores a session key in the storage
    async fn store(&self, request: Self::WriteReq) -> Result<Self::WriteRes>;
    /// Reads a session key from the storage based on the provided request
    async fn read(&self, request: Self::ReadReq) -> Result<Self::ReadRes>;
}

#[async_trait]
pub trait LastFMStorage:
    Storage<
        ReadReq = KeyReadRequest,
        ReadRes = KeyReadResult,
        WriteReq = LastFMSessionKey,
        WriteRes = KeyWriteResult,
    >
{
    async fn get_synced_tracks(&self) -> Result<Box<Vec<String>>>;
    async fn mark_tracks_as_synced(&self, track_ids: Vec<String>) -> Result<()>;
    async fn read_auth_key(&self, request: KeyReadRequest) -> Result<KeyReadResult> {
        self.read(request).await
    }
    async fn store_auth_key(&self, request: LastFMSessionKey) -> Result<KeyWriteResult> {
        self.store(request).await
    }
}
