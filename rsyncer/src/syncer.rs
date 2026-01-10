use futures::stream::{StreamExt, iter};
use log::{debug, info, warn};
use rsyncer::clients::LocalStorage;
use rsyncer::clients::{
    errors::{Error, Result},
    lastfm::LastFmClient,
    spotify::SpotifyClient,
};
use std::collections::HashSet;
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
        Ok(Config { spotify, lastfm, storage, concurrency: self.concurrency.unwrap_or(10) })
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
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

    /// Synchronizes liked tracks from Spotify to Last.fm
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

        info!("{} tracks have already been processed", processed_track_ids.len());

        // Identify unprocessed tracks by using their IDs and local storage
        let processed_set: HashSet<_> = processed_track_ids.into_iter().collect();
        let unprocessed_tracks: Vec<_> =
            tracks.into_iter().filter(|t| !processed_set.contains(&t.id)).collect();

        let lastfm = &self.config.lastfm;

        // Mark tracks as loved on LastFM concurrently

        let concurrency = self.config.concurrency; // Use concurrency from config

        let sync_results = iter(unprocessed_tracks)
            .map(|t| async move {
                if !lastfm.track_exists(&t).await? {
                    return Err(Error::UnknownTrack(t.id));
                }
                lastfm.love_track(&t).await?;
                Ok(t.id)
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

        let unprocessed_track_ids_len = unprocessed_track_ids.len();

        // Mark tracks as synced in local storage to avoid reprocessing them in future runs
        self.config.storage.mark_tracks_as_synced(unprocessed_track_ids).await?;
        info!("Sync process completed successfully. Synced tracks: {unprocessed_track_ids_len:?}",);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsyncer::clients::entities::{Artist, Track};

    fn create_test_track(id: &str, name: &str, artist: &str) -> Track {
        Track {
            id: id.to_string(),
            name: name.to_string(),
            artist: Artist { name: artist.to_string() },
        }
    }

    #[test]
    fn test_config_builder_new() {
        let builder = ConfigBuilder::new();
        assert!(builder.spotify.is_none());
        assert!(builder.lastfm.is_none());
        assert!(builder.storage.is_none());
        assert!(builder.concurrency.is_none());
    }

    #[test]
    fn test_config_builder_default_trait() {
        let builder = ConfigBuilder::default();
        assert!(builder.spotify.is_none());
        assert!(builder.lastfm.is_none());
        assert!(builder.storage.is_none());
        assert!(builder.concurrency.is_none());
    }

    #[test]
    fn test_track_creation() {
        let track = create_test_track("track1", "Song Name", "Artist Name");
        assert_eq!(track.id, "track1");
        assert_eq!(track.name, "Song Name");
        assert_eq!(track.artist.name, "Artist Name");
    }

    // Helper function that mirrors the sync logic for filtering unprocessed tracks
    fn filter_unprocessed_tracks(all_tracks: Vec<Track>, processed_ids: Vec<String>) -> Vec<Track> {
        let processed_set: HashSet<_> = processed_ids.into_iter().collect();
        all_tracks.into_iter().filter(|t| !processed_set.contains(&t.id)).collect()
    }

    #[test]
    fn test_filter_unprocessed_tracks_with_no_processed() {
        let tracks = vec![
            create_test_track("1", "Track 1", "Artist 1"),
            create_test_track("2", "Track 2", "Artist 2"),
        ];
        let processed: Vec<String> = vec![];

        let result = filter_unprocessed_tracks(tracks, processed);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_unprocessed_tracks_with_all_processed() {
        let tracks = vec![
            create_test_track("1", "Track 1", "Artist 1"),
            create_test_track("2", "Track 2", "Artist 2"),
        ];
        let processed = vec!["1".to_string(), "2".to_string()];

        let result = filter_unprocessed_tracks(tracks, processed);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_unprocessed_tracks_with_partial_processed() {
        let tracks = vec![
            create_test_track("1", "Track 1", "Artist 1"),
            create_test_track("2", "Track 2", "Artist 2"),
            create_test_track("3", "Track 3", "Artist 3"),
        ];
        let processed = vec!["2".to_string()];

        let result = filter_unprocessed_tracks(tracks, processed);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "1");
        assert_eq!(result[1].id, "3");
    }

    #[test]
    fn test_filter_unprocessed_tracks_with_duplicates() {
        let tracks = vec![
            create_test_track("1", "Track 1", "Artist 1"),
            create_test_track("2", "Track 2", "Artist 2"),
        ];
        // HashSet will deduplicate automatically
        let processed = vec!["1".to_string(), "1".to_string()];

        let result = filter_unprocessed_tracks(tracks, processed);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "2");
    }

    #[test]
    fn test_filter_unprocessed_tracks_empty_input() {
        let tracks: Vec<Track> = vec![];
        let processed = vec!["1".to_string()];

        let result = filter_unprocessed_tracks(tracks, processed);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_unprocessed_tracks_preserves_order() {
        let tracks = vec![
            create_test_track("1", "Track 1", "Artist 1"),
            create_test_track("2", "Track 2", "Artist 2"),
            create_test_track("3", "Track 3", "Artist 3"),
            create_test_track("4", "Track 4", "Artist 4"),
        ];
        let processed = vec!["2".to_string(), "4".to_string()];

        let result = filter_unprocessed_tracks(tracks, processed);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "1");
        assert_eq!(result[1].id, "3");
    }
}
