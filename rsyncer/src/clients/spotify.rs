use std::path::PathBuf;

use log::debug;

use crate::clients::{
    entities::{Artist, Track},
    errors::Error,
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
            artist: Artist {
                name: f.track.artists[0].name.clone(),
            },
            url: "test".to_string(),
        }
    }
}

pub struct SpotifyClient {
    pub spotify: AuthCodeSpotify, // TODO: test generic client
}

impl SpotifyClient {
    pub fn new(spotify: AuthCodeSpotify) -> Self {
        SpotifyClient { spotify }
    }
    pub fn test(&self) {
        println!("SpotifyClient test");
    }

    // Fetch tracks from Spotify Liked Songs default playlist
    pub async fn get_liked_tracks(&self) -> Result<Box<[Track]>, Error> {
        let mut tracks = vec![];

        // Obtaining the access token

        let mut stream = self.spotify.current_user_saved_tracks(None);

        // .current_user_top_tracks_manual(None, Some(10), Some(0))
        // let mut stream = spotify.current_user_top_tracks(None);

        while let Some(track) = stream.try_next().await? {
            tracks.push(Track::from(track));
            break; // TODO: remove break to fetch all tracks
        }

        Ok(tracks.into_boxed_slice())
        // Ok(tracks
        //     .into_iter()
        //     .take(5)
        //     .collect::<Vec<_>>()
        //     .into_boxed_slice())
    }

    // Authorize the Spotify client via CLI prompt and OAuth flow
    // This function requires the `cli` feature enabled.
    pub async fn authorize_client(&self) -> Result<(), Error> {
        let url = self.spotify.get_authorize_url(false)?;
        // This function requires the `cli` feature enabled.
        self.spotify.prompt_for_token(&url).await?;
        let user = self.spotify.me().await?;
        debug!("Authenticated as user: {:?}", user.display_name);
        Ok(())
    }

    // Create a SpotifyClient from environment variables or raise a configuration error
    pub fn try_default() -> Result<Self, Error> {
        let creds = Credentials::from_env()
        .ok_or_else(|| Error::ConfigurationError("Missing Spotify credentials in environment variables. Check README.MD for details.".into()))?;
        let oauth = OAuth::from_env(scopes!("user-top-read", "user-library-read"))
        .ok_or_else(|| Error::ConfigurationError("Missing Spotify OAuth configuration in environment variables. Check README.MD for details.".into()))?;

        // Set up token caching in a default cache directory
        let cache_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp")) // Fallback to /tmp if cache directory can't be determined
            .join(".rsyncer_cache");

        let spotify = AuthCodeSpotify::with_config(
            creds,
            oauth,
            Config {
                token_cached: true,
                cache_path,
                ..Default::default()
            },
        );

        Ok(Self { spotify })
    }
}
