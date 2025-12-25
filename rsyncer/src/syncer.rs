use futures::stream::{StreamExt, iter};
use log::{debug, info, warn};
use rsyncer::clients::LocalStorage;
use rsyncer::clients::{
    errors::{Error, Result},
    lastfm::LastFmClient,
    spotify::SpotifyClient,
};
use std::sync::Arc;

// Configuration for the Syncer Struct
pub struct Config {
    pub spotify: SpotifyClient,
    pub lastfm: LastFmClient,
    pub storage: Arc<LocalStorage>,
    pub concurrency: usize,
}

pub struct ConfigBuilder {
    spotify: Option<SpotifyClient>,
    lastfm: Option<LastFmClient>,
    storage: Option<Arc<LocalStorage>>,
    concurrency: Option<usize>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            spotify: None,
            lastfm: None,
            storage: None,
            concurrency: None, // Default concurrency for sync calls to LastFM API. Default is 10.
        }
    }

    pub async fn build(self) -> Result<Config> {
        let spotify = match self.spotify {
            Some(s) => s,
            None => SpotifyClient::try_default()?,
        };
        let storage = match self.storage {
            Some(s) => s,
            None => Arc::new(LocalStorage::try_default().await?),
        };
        let lastfm = match self.lastfm {
            Some(l) => l,
            None => LastFmClient::try_default(storage.clone())?,
        };
        Ok(Config {
            spotify,
            lastfm,
            storage,
            concurrency: self.concurrency.unwrap_or(10),
        })
    }
}

// The main Syncer struct that performs the synchronization
pub struct Syncer {
    config: Config,
}

impl Syncer {
    pub fn new(config: Config) -> Self {
        Syncer { config }
    }

    pub async fn sync(&self) -> Result<()> {
        info!("Starting sync process ...");
        debug!("Fetching liked tracks from Spotify ...");
        let tracks = self.config.spotify.get_liked_tracks().await?;
        debug!("Fetched {} liked tracks from Spotify", tracks.len());

        if tracks.is_empty() {
            info!("No liked tracks found on Spotify. Sync process completed.");
            return Ok(());
        }

        // Filter out already processed tracks
        let processed_track_ids: Vec<_> = self.config.storage.get_synced_tracks().await?;

        info!(
            "{} tracks have already been processed",
            processed_track_ids.len()
        );

        // Identify unprocessed tracks by using their IDs and local storage
        let unprocessed_tracks: Vec<_> = tracks
            .into_iter()
            .filter(|t| !processed_track_ids.contains(&t.id))
            .collect();

        let lastfm = &self.config.lastfm;

        // Mark tracks as loved on LastFM concurrently

        let concurrency = self.config.concurrency; // Use concurrency from config

        let sync_results = iter(unprocessed_tracks)
            .map(|t| async move {
                match lastfm.track_exists(&t).await {
                    Ok(exists) => {
                        if exists {
                            match lastfm.love_track(&t).await {
                                Ok(_) => Ok(t.id),
                                Err(e) => Err(e),
                            }
                        } else {
                            Err(Error::UnknownTrack(t.id))
                        }
                    }
                    Err(e) => Err(e),
                }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<Result<String>>>()
            .await;

        // Collect IDs that were synced successfully with LastFM
        let unprocessed_track_ids = sync_results
            .into_iter()
            .filter_map(|res| match res {
                Ok(id) => Some(id),
                Err(e) => {
                    warn!("Error processing track: {e:?}");
                    None
                }
            })
            .collect::<Vec<_>>();

        // Mark tracks as synced in local storage to avoid reprocessing them in future runs
        self.config
            .storage
            .mark_tracks_as_synced(unprocessed_track_ids.clone())
            .await?;

        info!(
            "Sync process completed successfully. Synced tracks: {:?}",
            unprocessed_track_ids.len()
        );
        Ok(())
    }
}
