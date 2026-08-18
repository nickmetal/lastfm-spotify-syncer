use async_duckdb::ClientBuilder;
use async_duckdb::duckdb::OptionalExt;
use log::debug;
use log::info;
use rsyncer_core::errors::Error::StorageError;
use rsyncer_core::storage::LastFMStorage;
use std::path::PathBuf;

use async_trait::async_trait;
use rsyncer_core::clients::errors::{Error, Result};
use rsyncer_core::clients::storage::KeyReadRequest;
use rsyncer_core::clients::storage::KeyReadResult;
use rsyncer_core::clients::storage::KeyWriteResult;
use rsyncer_core::clients::storage::LastFMSessionKey;
use rsyncer_core::clients::storage::Storage;

// Default DB user identifier for session key storage
// This is a single-user application.
// TODO: fix this
const DEFAULT_USER: &str = "default";

enum Table {
    LastFMSession,
    SyncedTrack,
}

impl Table {
    pub fn as_str(&self) -> &'static str {
        match self {
            Table::LastFMSession => "last_fm_session",
            Table::SyncedTrack => "synced_track",
        }
    }
}

/// Local storage client using `DuckDB` for caching and persistence
///
/// Stores Last.fm session keys and tracks that have been synced to avoid
/// reprocessing them on subsequent runs.
pub struct LocalStorage {
    client: async_duckdb::Client,
}

impl LocalStorage {
    /// Creates a new `LocalStorage` instance with the provided `DuckDB` client
    #[must_use]
    pub fn new(client: async_duckdb::Client) -> Self {
        LocalStorage { client }
    }

    /// Initializes the database by creating necessary tables and sequences
    ///
    /// Creates:
    /// - `last_fm_session` table for storing session keys
    /// - `synced_track` table for tracking processed tracks
    /// - An ID sequence for potential future use
    pub async fn init(&self) -> Result<()> {
        // Create necessary tables that Rsyncer will use
        let seq_name = "id_sequence";
        let table_query = format!(
            "
            CREATE SEQUENCE IF NOT EXISTS {seq} START 1;
            CREATE TABLE IF NOT EXISTS {session_table} (
                user TEXT PRIMARY KEY DEFAULT '{DEFAULT_USER}',
                session_key TEXT
            );
            CREATE TABLE IF NOT EXISTS {track_table} (
                track_id TEXT PRIMARY KEY
            );
        ",
            seq = seq_name,
            session_table = Table::LastFMSession.as_str(),
            track_table = Table::SyncedTrack.as_str()
        );
        self.client
            .conn(move |conn| conn.execute_batch(&table_query))
            .await
            .map_err(|err: async_duckdb::Error| Error::StorageError(err.to_string()))?;

        debug!("Successfully initialized local storage database");
        Ok(())
    }

    /// Creates a `LocalStorage` instance using the default cache directory
    ///
    /// The database file is stored at `~/.cache/.rsyncer_db.duckdb` (or `/tmp` as fallback).
    pub async fn try_default() -> Result<Self> {
        let db_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp")) // Fallback to /tmp if cache directory can't be determined
            .join(".rsyncer_db.duckdb");
        let client: async_duckdb::Client = ClientBuilder::new()
            .path(&db_path)
            .open()
            .await
            .map_err(|err: async_duckdb::Error| Error::StorageError(err.to_string()))?;
        debug!("Opened local storage database at {}", db_path.display());
        Ok(LocalStorage { client })
    }

    /// Reads the cached Last.fm session key from local storage
    ///
    /// Returns `None` if no session key is stored or if an error occurs.
    async fn read_session_key(&self) -> Result<Option<String>> {
        let query = format!(
            "SELECT session_key FROM {} WHERE user = '{DEFAULT_USER}';",
            Table::LastFMSession.as_str()
        );

        let key = self
            .client
            .conn(move |conn| conn.query_row(&query, [], |row| row.get(0).optional()))
            .await
            .map_err(|err| StorageError(err.to_string()))?;

        Ok(key)
    }

    /// Stores or updates the Last.fm session key in local storage
    ///
    /// Uses a MERGE statement to insert or update the session key for the default user.
    async fn update_session_key(&self, key: String) -> Result<()> {
        let table = Table::LastFMSession.as_str();
        let key_escaped = key.replace('\'', "''");
        let query = format!(
            "
            MERGE INTO {table}
            USING (
                SELECT unnest(['{DEFAULT_USER}']) AS user,
                       unnest(['{key_escaped}']) AS session_key
            ) AS upserts
            ON (upserts.user = {table}.user)
            WHEN MATCHED THEN UPDATE
            WHEN NOT MATCHED THEN INSERT;
            "
        );

        self.client
            .conn(move |conn| conn.execute_batch(&query))
            .await
            .map_err(|err: async_duckdb::Error| Error::StorageError(err.to_string()))?;

        debug!("Update session key in local storage using MERGE INTO");
        Ok(())
    }
}

#[async_trait]
impl Storage for LocalStorage {
    type ReadReq = KeyReadRequest;
    type ReadRes = KeyReadResult;
    type WriteReq = LastFMSessionKey;
    type WriteRes = KeyWriteResult;

    async fn store(&self, request: Self::WriteReq) -> Result<Self::WriteRes> {
        self.update_session_key(request.key).await?;
        Ok(KeyWriteResult {})
    }

    async fn read(&self, _request: Self::ReadReq) -> Result<Self::ReadRes> {
        let key_opt = self.read_session_key().await?;
        match key_opt {
            Some(key) => Ok(KeyReadResult::Found(LastFMSessionKey { key })),
            None => Ok(KeyReadResult::NotFound),
        }
    }
}

#[async_trait]
impl LastFMStorage for LocalStorage {
    /// Fetches all synced track IDs from the local storage
    ///
    /// Returns a vector of track IDs that have been previously processed.
    async fn get_synced_tracks(&self) -> Result<Box<Vec<String>>> {
        let query = format!("SELECT track_id FROM {};", Table::SyncedTrack.as_str());

        let track_ids = self
            .client
            .conn(move |conn| {
                let mut stmt = conn.prepare(&query)?;
                let mut rows = stmt.query([])?;
                let mut ids = vec![];
                while let Some(row) = rows.next()? {
                    let id: String = row.get(0)?;
                    ids.push(id);
                }
                Ok(ids)
            })
            .await
            .map_err(|err: async_duckdb::Error| Error::StorageError(err.to_string()))?;

        Ok(Box::new(track_ids))
    }

    /// Adds track IDs to the synced tracks table to mark them as processed
    ///
    /// # Warning
    /// This method may not add records if any of the track IDs already exist in the database.
    async fn mark_tracks_as_synced(&self, track_ids: Vec<String>) -> Result<()> {
        if track_ids.is_empty() {
            debug!("No tracks to mark as synced");
            return Ok(());
        }

        let res = self
            .client
            .conn(move |conn| {
                // TODO: try remove move: into iter instead of iter
                let params: Vec<[&str; 1]> =
                    track_ids.iter().map(move |id| [id.as_str()]).collect();

                info!("Marking {} tracks as synced in local storage", track_ids.len());
                let mut app: async_duckdb::duckdb::Appender<'_> =
                    conn.appender(Table::SyncedTrack.as_str())?;
                app.append_rows(&params)?;

                Ok(())
            })
            .await;

        match res {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::StorageError(e.to_string())),
        }
    }
}
