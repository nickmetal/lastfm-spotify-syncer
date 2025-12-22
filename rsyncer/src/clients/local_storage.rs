use async_duckdb::ClientBuilder;
use async_duckdb::Error as DuckDBError;
use async_duckdb::duckdb::AppenderParams;
use async_duckdb::duckdb::OptionalExt;
use async_duckdb::duckdb::params;
use log::debug;
use std::path::PathBuf;

use crate::clients::errors::Error;

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

pub struct LocalStorage {
    client: async_duckdb::Client,
}

impl LocalStorage {
    pub fn new(client: async_duckdb::Client) -> Self {
        LocalStorage { client }
    }

    pub async fn init_db(&self) -> Result<(), Error> {
        // Create necessary tables that Rsyncer will use
        let seq_name = "id_sequence";
        // id INTEGER PRIMARY KEY DEFAULT nextval('{seq}'),
        let table_query = format!(
            "
            CREATE SEQUENCE IF NOT EXISTS {seq} START 1;
            CREATE TABLE IF NOT EXISTS {session_table} (
                id INTEGER PRIMARY KEY DEFAULT nextval('{seq}'),
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
            .await?;

        debug!("Successfully initialized local storage database");
        Ok(())
    }

    pub async fn try_default() -> Result<Self, Error> {
        let db_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp")) // Fallback to /tmp if cache directory can't be determined
            .join(".rsyncer_db.duckdb");
        let client: async_duckdb::Client = ClientBuilder::new().path(&db_path).open().await?;
        debug!("Opened local storage database at {db_path:?}");
        Ok(LocalStorage { client })
    }

    pub async fn read_session_key(&self) -> Option<String> {
        let query = format!(
            "SELECT session_key FROM {} ORDER BY id DESC LIMIT 1;",
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

    pub async fn store_session_key(&self, key: &str) -> Result<(), Error> {
        let query = format!(
            "INSERT INTO {} (session_key) VALUES (?1);",
            Table::LastFMSession.as_str()
        );
        let key_owned = key.to_string();

        self.client
            .conn(move |conn| conn.execute(&query, [key_owned.clone()]))
            .await?;

        debug!("Stored new session key in local storage");
        Ok(())
    }

    pub async fn update_session_key(&self, key: &str) -> Result<(), Error> {
        let query = format!(
            "UPDATE {} SET session_key = ?1;",
            Table::LastFMSession.as_str()
        );
        let key_owned = key.to_string();

        self.client
            .conn(move |conn| conn.execute(&query, [key_owned.clone()]))
            .await?;

        debug!("Update session key in local storage");
        Ok(())
    }

    // Check if a track ID exists in the synced tracks table
    pub async fn is_track_synced(&self, track_id: &str) -> Result<bool, Error> {
        let query = format!(
            "SELECT 1 FROM {} WHERE track_id = ?1 LIMIT 1;",
            Table::SyncedTrack.as_str()
        );
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
            Err(_) => todo!(),
        }
    }

    // Adds track IDs to the synced tracks table
    // WARNING: This method doesn't add any records if at least one of the track IDs already exists in db
    pub async fn mark_tracks_as_synced(&self, track_ids: Box<[String]>) -> Result<(), Error> {
        let res = self
            .client
            .conn(move |conn| {
                let params: Vec<[&str; 1]> =
                    track_ids.iter().map(move |id| [id.as_str()]).collect();
                debug!("Marking {:?} tracks as synced", params.clone());
                let mut app: async_duckdb::duckdb::Appender<'_> =
                    conn.appender(Table::SyncedTrack.as_str())?;
                app.append_rows(&params)?;

                Ok(())
            })
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::StorageError(e)),
        }
    }

    // Fetch all synced track IDs from the local storage
    pub async fn get_synced_tracks(&self) -> Result<Vec<String>, Error> {
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
