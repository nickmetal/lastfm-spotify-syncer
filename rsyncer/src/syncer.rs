use std::default;

use futures::future::{join, join_all};
use futures::stream::{StreamExt, iter};
use log::{debug, info, warn};
use rsyncer::clients::LocalStorage;
use rsyncer::clients::{errors::Error, lastfm::LastFmClient, spotify::SpotifyClient};

// Configuration for the Syncer Struct
pub struct Config {
    pub spotify: SpotifyClient,
    pub lastfm: LastFmClient,
    pub storage: LocalStorage,
}

pub struct ConfigBuilder {
    spotify: Option<SpotifyClient>,
    lastfm: Option<LastFmClient>,
    storage: Option<LocalStorage>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            spotify: None,
            lastfm: None,
            storage: None,
        }
    }

    pub fn spotify(mut self, spotify: SpotifyClient) -> Self {
        self.spotify = Some(spotify);
        self
    }

    pub fn lastfm(mut self, lastfm: LastFmClient) -> Self {
        self.lastfm = Some(lastfm);
        self
    }

    pub fn storage(mut self, storage: LocalStorage) -> Self {
        self.storage = Some(storage);
        self
    }

    pub async fn build(self) -> Result<Config, Error> {
        let spotify = match self.spotify {
            Some(s) => s,
            None => SpotifyClient::try_default()?,
        };
        let lastfm = match self.lastfm {
            Some(l) => l,
            None => LastFmClient::try_default()?,
        };
        let storage = match self.storage {
            Some(s) => s,
            None => LocalStorage::try_default().await?,
        };
        Ok(Config {
            spotify,
            lastfm,
            storage,
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

    pub async fn sync(&self) -> Result<(), Error> {
        info!("Starting sync process ...");
        debug!("Fetching liked tracks from Spotify ...");
        let tracks = self.config.spotify.get_liked_tracks().await?;
        debug!("Fetched {} liked tracks from Spotify", tracks.len());

        let lastfm = &self.config.lastfm;
        let future_results = iter(tracks)
            .then(|t| async move {
                match lastfm.track_exists(&t).await {
                    Ok(exists) => {
                        if exists {
                            match self.config.lastfm.love_track(&t).await {
                                Ok(_) => {
                                    info!(
                                        "Successfully loved track {} by {} in Last.fm",
                                        t.name, t.artist.name
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        "Error loving track {} by {}: {:?}",
                                        t.name, t.artist.name, e
                                    );
                                }
                            }
                        } else {
                            warn!(
                                "Track {} by {} does not exist in Last.fm",
                                t.name, t.artist.name
                            );
                        }
                    }
                    Err(e) => {
                        warn!("Error checking track {t:?} {e:?}");
                    }
                };
            })
            .collect::<Vec<_>>()
            .await;

        info!("Sync process completed successfully.");
        Ok(())
    }
}
