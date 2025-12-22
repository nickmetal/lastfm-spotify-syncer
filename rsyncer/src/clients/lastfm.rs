use std::{collections::HashSet, path::PathBuf};

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::clients::{entities::Track, errors::Error};
// use dotenv::dotenv;
// use lastfm_rust::api::Track;
use lastfm_rust::{APIResponse, Error as LastFMError, Lastfm};
// use std::error::Error;

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
// Handles caching of Last.fm session key to avoid re-authentication for each run. It uses local file storage for simplicity.
// NOTE: this manager is for a single user application. Maybe extend later to multi-user?

// Result of loading cached auth session key from local storage
enum LastFMCachedAuthResult {
    Cached(String),
    NotFound,
    Error(Error),
}

struct LastFMCachedAuth {}

impl LastFMCachedAuth {
    fn new() -> Self {
        LastFMCachedAuth {}
    }

    pub async fn store_session_key(&self, key: String) -> Result<(), Error> {
        let cache_path = self.get_cache_file_path();
        tokio::fs::write(cache_path.clone(), key).await?;
        debug!("Stored Last.fm session key in cache in {cache_path:?}");
        Ok(())
    }
    pub async fn load_session_key(&self) -> LastFMCachedAuthResult {
        let cache_path = self.get_cache_file_path();
        match tokio::fs::try_exists(cache_path.clone()).await {
            Ok(exists) => {
                if exists {
                    match tokio::fs::read_to_string(cache_path).await {
                        Ok(contents) => {
                            debug!("Loaded Last.fm session key from cache");
                            LastFMCachedAuthResult::Cached(contents)
                        }
                        Err(e) => {
                            debug!("Failed to load Last.fm session key from cache: {e}");
                            LastFMCachedAuthResult::Error(Error::from(e))
                        }
                    }
                } else {
                    debug!("No cached Last.fm session key found in {cache_path:?}");
                    LastFMCachedAuthResult::NotFound
                }
            }
            Err(e) => {
                debug!("Error checking for Last.fm session key cache: {e}");
                LastFMCachedAuthResult::Error(Error::from(e))
            }
        }
    }

    fn get_cache_file_path(&self) -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp")) // Fallback to /tmp if cache directory can't be determined
            .join(".rsyncer_lfm_session_cache")
    }
}

pub struct LastFmClient {
    lastfm: Lastfm,
    cached_auth: LastFMCachedAuth,
}

impl LastFmClient {
    pub fn new(lastfm: Lastfm) -> Self {
        LastFmClient {
            lastfm,
            cached_auth: LastFMCachedAuth::new(),
        }
    }
    pub fn try_default() -> Result<Self, Error> {
        let api_key = std::env::var("LASTFM_API_KEY")?;
        let api_secret = std::env::var("LASTFM_API_SECRET")?;

        let lastfm = Lastfm::builder()
            .api_key(api_key)
            .api_secret(api_secret)
            .build()?;

        Ok(LastFmClient::new(lastfm))
    }

    pub async fn get_session_key_from_api(&self) -> Result<String, Error> {
        let response = self.lastfm.auth().get_token().send().await?;
        let token = match response {
            APIResponse::Success(value) => value.token,
            APIResponse::Error(err) => {
                return Err(Error::LastFMError(LastFMError::ApiError(err)));
            }
        };
        // Authorize the token
        self.lastfm.auth().pls_authorize(token.to_string());
        // Get session key
        let response = self
            .lastfm
            .auth()
            .get_session()
            .token(&token)
            .send()
            .await?;

        let auth_response: AuthSessionResponse = serde_json::from_value(response)?;
        Ok(auth_response.session.key)
    }

    // Authorize the client by obtaining a session key
    // All calls that require authentication will use this session key
    pub async fn authorize_client(&mut self) -> Result<(), Error> {
        // Request token
        let session_key_result = self.cached_auth.load_session_key().await;
        match session_key_result {
            LastFMCachedAuthResult::Cached(session_key) => {
                self.lastfm.set_sk(session_key);
                // validate session key. Key may be invalid if user revoked access or by other reasons
                match self.lastfm.user().get_info().send().await {
                    Ok(APIResponse::Success(_)) => {
                        debug!("Loaded valid Last.fm session key from cache");
                    }
                    Ok(APIResponse::Error(err)) => {
                        debug!("Cached Last.fm session key is invalid: {err:?}, re-authenticating");
                        // Invalidate cached session key and re-authenticate
                        let session_key = self.get_session_key_from_api().await?;
                        self.lastfm.set_sk(session_key.clone());
                        // Store session key in cache
                        self.cached_auth.store_session_key(session_key).await?;
                    }
                    Err(err) => {
                        debug!("Cached Last.fm session key is invalid: {err:?}, re-authenticating");
                        // Invalidate cached session key and re-authenticate
                        let session_key = self.get_session_key_from_api().await?;
                        self.lastfm.set_sk(session_key.clone());
                        // Store session key in cache
                        self.cached_auth.store_session_key(session_key).await?;
                    }
                }
            }
            LastFMCachedAuthResult::NotFound => {
                // No cached session key found, create a new one and store it

                let session_key = self.get_session_key_from_api().await?;
                self.lastfm.set_sk(session_key.clone());
                // Store session key in cache
                self.cached_auth.store_session_key(session_key).await?;
            }
            LastFMCachedAuthResult::Error(err) => return Err(err),
        }
        Ok(())
    }

    pub async fn track_exists(&self, track: &Track) -> Result<(bool), Error> {
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
                let distinct_artist_names: HashSet<String> = HashSet::from_iter(
                    response
                        .results
                        .trackmatches
                        .track
                        .iter()
                        .map(|t| t.artist.clone()),
                );
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

    pub async fn love_track(&self, track: &Track) -> Result<(), Error> {
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
