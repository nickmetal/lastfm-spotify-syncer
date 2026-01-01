use rspotify::ClientError;
use thiserror::Error;

/// Error types for the rsyncer application
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to parse or transmit data
    #[error("Failed to parse transmit data, error: {0}")]
    ParseError(String),

    // Disconnect(#[from] io::Error),
    /// Error from the Last.fm API client
    #[error("LastFM error: {0}")]
    LastFMError(#[from] lastfm_rust::Error),

    /// Failed to deserialize Last.fm API response
    #[error("LastFM Deserialization error: {0}")]
    LastFMDeserializationError(#[from] serde_json::Error),

    /// Last.fm API returned an unexpected response format
    #[error("LastFM API unexpected response: {0}")]
    LastFMUnexpectedResponse(String),

    /// Error from the Spotify API client
    #[error("Spotify error: {0}")]
    SpotifyError(#[from] ClientError),

    /// Configuration error (missing env vars, invalid settings, etc.)
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Error from the local `DuckDB` storage
    #[error("Storage error: {0}")]
    StorageError(#[from] async_duckdb::Error),

    /// Track not found on Last.fm
    #[error("Unknown Track: {0}")]
    UnknownTrack(String),
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Error::ConfigurationError(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::ConfigurationError(err.to_string())
    }
}

/// Result type alias using the crate's Error type
pub type Result<T> = core::result::Result<T, Error>;
