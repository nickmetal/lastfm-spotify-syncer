use std::path::PathBuf;

use log::debug;

use crate::clients::{
    entities::{Artist, Track},
    errors::{Error, Result},
};
use futures::stream::TryStreamExt;
use rspotify::{
    AuthCodeSpotify, Config, Credentials, OAuth, model::SavedTrack, prelude::*, scopes,
};

impl From<SavedTrack> for Track {
    fn from(f: SavedTrack) -> Track {
        Track {
            id: f.track.id.unwrap().to_string(),
            name: f.track.name,
            artist: Artist { name: f.track.artists[0].name.clone() },
        }
    }
}

/// A client for interacting with the Spotify API, handling authentication and track retrieval.
pub struct SpotifyClient {
    /// The underlying authenticated Spotify API client.
    pub spotify: AuthCodeSpotify,
}

impl SpotifyClient {
    /// Creates a new `SpotifyClient` with the given `AuthCodeSpotify` instance.
    #[must_use]
    pub fn new(spotify: AuthCodeSpotify) -> Self {
        SpotifyClient { spotify }
    }

    /// Fetches all liked tracks from the user's Spotify "Liked Songs" playlist
    ///
    /// Returns a vector of tracks with their IDs, names, and artist information.
    pub async fn get_liked_tracks(&self) -> Result<Vec<Track>> {
        let stream = self.spotify.current_user_saved_tracks(None);
        let tracks: Vec<Track> = stream.map_ok(Track::from).try_collect().await?;
        Ok(tracks)
    }

    /// Authorizes the Spotify client via CLI prompt and OAuth flow
    ///
    /// This will open a browser window for the user to log in and authorize the application.
    /// Requires the `cli` feature to be enabled.
    pub async fn authorize_client(&self) -> Result<()> {
        debug!("Starting Spotify authorization ...");
        let url = self.spotify.get_authorize_url(false)?;
        // This function requires the `cli` feature enabled.
        self.spotify.prompt_for_token(&url).await?;
        let user = self.spotify.me().await?;
        debug!("Authenticated as user: {:?}", user.display_name);
        Ok(())
    }

    /// Creates a `SpotifyClient` from environment variables or returns a configuration error if required variables are missing.
    pub fn try_default() -> Result<Self> {
        let creds = Credentials::from_env()
        .ok_or_else(|| Error::ConfigurationError("Missing Spotify credentials in environment variables. Check README.MD for details.".into()))?;
        let oauth = OAuth::from_env(scopes!("user-top-read", "user-library-read"))
        .ok_or_else(|| Error::ConfigurationError("Missing Spotify OAuth configuration in environment variables. Check README.MD for details.".into()))?;

        // Set up token caching in a default cache directory
        // TODO: check for duckdb usage here
        let cache_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp")) // Fallback to /tmp if cache directory can't be determined
            .join(".rsyncer_cache");

        let spotify = AuthCodeSpotify::with_config(
            creds,
            oauth,
            Config { token_cached: true, cache_path, ..Default::default() },
        );

        Ok(Self { spotify })
    }
}
