use async_duckdb::ClientBuilder;
use async_duckdb::Error as DuckDBError;
use async_duckdb::duckdb::OptionalExt;
use log::debug;
use log::info;
use std::path::PathBuf;

use crate::clients::errors::{Error, Result};

// Default DB user identifier for session key storage
// This is a single-user application.
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
    pub async fn init_db(&self) -> Result<()> {
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
        self.client.conn(move |conn| conn.execute_batch(&table_query)).await?;

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
        let client: async_duckdb::Client = ClientBuilder::new().path(&db_path).open().await?;
        debug!("Opened local storage database at {}", db_path.display());
        Ok(LocalStorage { client })
    }

    /// Reads the cached Last.fm session key from local storage
    ///
    /// Returns `None` if no session key is stored or if an error occurs.
    pub async fn read_session_key(&self) -> Option<String> {
        let query = format!(
            "SELECT session_key FROM {} WHERE user = '{DEFAULT_USER}';",
            Table::LastFMSession.as_str()
        );

        let key = self
            .client
            .conn(move |conn| conn.query_row(&query, [], |row| row.get(0).optional()))
            .await;

        match key {
            Ok(opt) => opt,
            Err(e) => {
                debug!("Failed to read session key: {e:?}");
                None
            }
        }
    }

    /// Stores or updates the Last.fm session key in local storage
    ///
    /// Uses a MERGE statement to insert or update the session key for the default user.
    pub async fn update_session_key(&self, key: String) -> Result<()> {
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

        self.client.conn(move |conn| conn.execute_batch(&query)).await?;

        debug!("Update session key in local storage using MERGE INTO");
        Ok(())
    }

    /// Checks if a track ID exists in the synced tracks table
    ///
    /// Returns `true` if the track has been previously synced, `false` otherwise.
    pub async fn is_track_synced(&self, track_id: &str) -> Result<bool> {
        let query =
            format!("SELECT 1 FROM {} WHERE track_id = ?1 LIMIT 1;", Table::SyncedTrack.as_str());
        let track_id_owned = track_id.to_string();

        let exists = self
            .client
            .conn(move |conn| {
                conn.query_row(&query, [track_id_owned.clone()], |row| {
                    row.get::<_, i32>(0).optional()
                })
            })
            .await;

        match exists {
            Ok(opt) => Ok(opt.is_some()),
            Err(DuckDBError::Duckdb(e)) => match e {
                async_duckdb::duckdb::Error::QueryReturnedNoRows => Ok(false),
                _ => Err(Error::StorageError(DuckDBError::Duckdb(e))),
            },
            Err(_) => {
                Err(Error::ConfigurationError("Failed to check if track is synced".to_string()))
            }
        }
    }

    /// Adds track IDs to the synced tracks table to mark them as processed
    ///
    /// # Warning
    /// This method may not add records if any of the track IDs already exist in the database.
    pub async fn mark_tracks_as_synced(&self, track_ids: Vec<String>) -> Result<()> {
        if track_ids.is_empty() {
            debug!("No tracks to mark as synced");
            return Ok(());
        }

        let res = self
            .client
            .conn(move |conn| {
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
            Err(e) => Err(Error::StorageError(e)),
        }
    }

    /// Fetches all synced track IDs from the local storage
    ///
    /// Returns a vector of track IDs that have been previously processed.
    pub async fn get_synced_tracks(&self) -> Result<Vec<String>> {
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
            .await?;

        Ok(track_ids)
    }
}
