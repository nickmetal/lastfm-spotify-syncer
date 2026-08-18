use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;
use log::debug;
use log::info;
use rsyncer_core::clients::errors::Result;
use rsyncer_core::clients::storage::KeyReadRequest;
use rsyncer_core::clients::storage::KeyReadResult;
use rsyncer_core::clients::storage::KeyWriteResult;
use rsyncer_core::clients::storage::LastFMSessionKey;
use rsyncer_core::clients::storage::Storage;
use rsyncer_core::errors::Error::ConfigurationError;
use rsyncer_core::errors::Error::StorageError;
use rsyncer_core::storage::LastFMStorage;
use sqlx::Pool;
use sqlx::Postgres;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug)]
enum DBClient {
    Connected { db: Pool<Postgres> },
    NotConnected,
}

#[derive(Debug)]
struct LastFMSessionKeyDBRecord {
    session_key: String,
}

#[derive(Debug)]
struct SyncedTrackDBRecord {
    track_id: String,
}

pub struct APILastFMStorage {
    database_url: String,
    migrations_path: PathBuf,
    dbclient: DBClient,
}

impl APILastFMStorage {
    #[must_use]
    pub fn new(database_url: String, migrations_path: PathBuf) -> Self {
        APILastFMStorage { database_url, migrations_path, dbclient: DBClient::NotConnected }
    }

    pub async fn connect(&mut self) -> Result<()> {
        debug!("Initializing storage: connecting");
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.database_url.to_string())
            .await
            .map_err(|err| StorageError(err.to_string()))?;

        debug!("Initializing storage: running migrations");
        let m = Migrator::new(self.migrations_path.to_path_buf())
            .await
            .map_err(|err| StorageError(err.to_string()))?;
        m.run(&db).await.map_err(|err| StorageError(err.to_string()))?;
        debug!("Initializing storage: Migration applied successfully");

        self.dbclient = DBClient::Connected { db };
        Ok(())
    }

    async fn read_session_key(&self) -> Result<Option<String>> {
        debug!("Reading lastfm session key from db");
        // TODO: add multi user query
        let fetched_key = match &self.dbclient {
            DBClient::NotConnected => {
                Err(ConfigurationError("Client is not initialized".to_owned()))
            }
            DBClient::Connected { db } => {
                let fetched_key = sqlx::query_as!(
                    LastFMSessionKeyDBRecord,
                    r#"SELECT session_key FROM lastfm_session_key LIMIT 1"#
                )
                .fetch_optional(db)
                .await
                .map_err(|e| StorageError(e.to_string()))?
                .map(|v| v.session_key);
                Ok(fetched_key)
            }
        }?;
        Ok(fetched_key)
    }

    async fn update_session_key(&self, key: String) -> Result<()> {
        debug!("Updating session key in storage");
        match &self.dbclient {
            DBClient::NotConnected => {
                Err(ConfigurationError("Client is not initialized".to_owned()))
            }
            DBClient::Connected { db } => {
                let created_at = Utc::now().naive_utc();
                sqlx::query!(
                    r#"
                        INSERT INTO lastfm_session_key (session_key, created_at) VALUES ($1::text, $2::timestamp)
                    "#,
                    &key,
                    &created_at,
                )
                .execute(db)
                .await
                .map_err(|e| StorageError(e.to_string()))?;
                debug!("Key stored");
                Ok(())
            }
        }?;
        Ok(())
    }
}

#[async_trait]
impl Storage for APILastFMStorage {
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
impl LastFMStorage for APILastFMStorage {
    async fn get_synced_tracks(&self) -> Result<Box<Vec<String>>> {
        debug!("Reading synced track ids from db");
        let track_ids = match &self.dbclient {
            DBClient::NotConnected => {
                Err(ConfigurationError("Client is not initialized".to_owned()))
            }
            DBClient::Connected { db } => {
                let track_ids: Vec<String> =
                    sqlx::query_as!(SyncedTrackDBRecord, r#"SELECT track_id FROM synced_track"#)
                        .fetch_all(db)
                        .await
                        .map_err(|e| StorageError(e.to_string()))?
                        .into_iter()
                        .map(|r| r.track_id)
                        .collect();
                info!("PG storage: fetched cached track_ids: {}", track_ids.len());
                Ok(track_ids)
            }
        }?;
        Ok(Box::new(track_ids))
    }

    async fn mark_tracks_as_synced(&self, track_ids: Vec<String>) -> Result<()> {
        debug!("Storing synced track ids in db");
        let result = match &self.dbclient {
            DBClient::NotConnected => {
                Err(ConfigurationError("Client is not initialized".to_owned()))
            }
            DBClient::Connected { db } => {
                let created_at = Utc::now().naive_utc();

                let created_at_values = vec![created_at; track_ids.len()];

                let res = sqlx::query!(
                    r#"
                        INSERT INTO synced_track (track_id, created_at)
                        SELECT *
                        FROM UNNEST($1::text[], $2::timestamp[])
                    "#,
                    &track_ids,
                    &created_at_values,
                )
                .execute(db)
                .await
                .map_err(|e| StorageError(e.to_string()))?;
                debug!("Tracks inserted: {}", res.rows_affected());
                Ok(())
            }
        }?;
        Ok(result)
    }
}
