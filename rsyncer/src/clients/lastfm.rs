use log::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashSet;
use std::sync::Arc;

use crate::clients::{
    LocalStorage,
    entities::Track,
    errors::{Error, Result},
};
use lastfm_rust::{APIResponse, Error as LastFMError, Lastfm};

#[derive(Serialize, Deserialize, Debug)]
struct LastFMAPITrack {
    url: String,
    name: String,
    artist: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tracks {
    track: Vec<LastFMAPITrack>,
}
#[derive(Serialize, Deserialize, Debug)]
struct TrackMatches {
    trackmatches: Tracks,
}

#[derive(Serialize, Deserialize, Debug)]
struct TrackSearchResponse {
    results: TrackMatches,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthSession {
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthSessionResponse {
    session: AuthSession,
}

/// Client for interacting with the Last.fm API, handling authentication and track operations.
/// Client for interacting with the Last.fm API
pub struct LastFmClient {
    lastfm: Lastfm,
    storage: Arc<LocalStorage>,
}

impl LastFmClient {
    /// Creates a new Last.fm client with the provided API client and storage
    #[must_use]
    pub fn new(lastfm: Lastfm, storage: Arc<LocalStorage>) -> Self {
        LastFmClient { lastfm, storage }
    }

    /// Creates a Last.fm client using API credentials from environment variables
    ///
    /// Expects `LASTFM_API_KEY` and `LASTFM_API_SECRET` environment variables to be set.
    pub fn try_default(storage: Arc<LocalStorage>) -> Result<Self> {
        let api_key = std::env::var("LASTFM_API_KEY")?;
        let api_secret = std::env::var("LASTFM_API_SECRET")?;

        let lastfm = Lastfm::builder().api_key(api_key).api_secret(api_secret).build()?;
        Ok(LastFmClient::new(lastfm, storage))
    }

    /// Obtains a new session key from the Last.fm API via user authorization
    ///
    /// This will prompt the user to authorize the application in their browser.
    pub async fn get_session_key_from_api(&self) -> Result<String> {
        let response = self.lastfm.auth().get_token().send().await?;
        let token = match response {
            APIResponse::Success(value) => value.token,
            APIResponse::Error(err) => {
                return Err(Error::LastFMError(LastFMError::ApiError(err)));
            }
        };
        // Authorize the token
        self.lastfm.auth().pls_authorize(token.clone());
        // Get session key
        let response = self.lastfm.auth().get_session().token(&token).send().await?;

        let auth_response: AuthSessionResponse = serde_json::from_value(response)?;
        Ok(auth_response.session.key)
    }

    /// Authorizes the client by obtaining a session key
    ///
    /// Attempts to load a cached session key from local storage first.
    /// If not found, initiates the OAuth flow to obtain a new session key.
    /// All authenticated API calls will use this session key.
    pub async fn authorize_client(&mut self) -> Result<()> {
        // Get cached session key from local storage if available
        let session_key_result = self.storage.read_session_key().await;

        if let Some(session_key) = session_key_result {
            // TODO: add session key validation. Key may be invalid if user revoked access or by other reasons
            self.lastfm.set_sk(session_key);
            return Ok(());
        }

        let session_key_from_api = self.get_session_key_from_api().await?;
        self.lastfm.set_sk(session_key_from_api.clone());
        // Store session key in storage to avoid re-authentication next time
        self.storage.update_session_key(session_key_from_api).await?;
        Ok(())
    }

    /// Checks if a track exists on Last.fm by searching for it
    ///
    /// Returns `true` if exactly one matching track is found, or if multiple tracks
    /// by the same artist are found. Returns an error if tracks by different artists
    /// are found (ambiguous result).
    pub async fn track_exists(&self, track: &Track) -> Result<bool> {
        let mut track_api = self.lastfm.track();
        let search_response = track_api
            .search()
            .artist(track.artist.name.as_str())
            .track(track.name.as_str())
            .limit(2) // expect only 1 track, if we get more - raise an error for now, TODO: handle similar Artist names
            .send()
            .await?;

        let response: TrackSearchResponse = match search_response {
            lastfm_rust::APIResponse::Success(json_content) => {
                serde_json::from_value(json_content)?
            }
            lastfm_rust::APIResponse::Error(err) => {
                return Err(Error::LastFMError(lastfm_rust::Error::ApiError(err)));
            }
        };

        match response.results.trackmatches.track.len() {
            0 => Ok(false),
            1 => Ok(true),
            _ => {
                // Collect distinct artist names specified in track artist field
                let distinct_artist_names: HashSet<String> =
                    response.results.trackmatches.track.iter().map(|t| t.artist.clone()).collect();
                // If all found tracks are by the same artist, consider it exists and it is valid candidate
                if distinct_artist_names.len() == 1 {
                    Ok(true)
                } else {
                    Err(Error::LastFMUnexpectedResponse(format!(
                        "Multiple tracks found for {} - {} by different artists: {:?}",
                        track.name, track.artist.name, distinct_artist_names
                    )))
                }
            }
        }
    }

    /// Marks a track as "loved" on Last.fm
    ///
    /// Requires the client to be authorized before calling.
    pub async fn love_track(&self, track: &Track) -> Result<()> {
        info!("Marking track as loved on Last.fm: {} - {}", track.artist.name, track.name);
        self.lastfm
            .track()
            .love()
            .artist(track.artist.name.as_str())
            .track(track.name.as_str())
            .send()
            .await?;
        Ok(())
    }
}
