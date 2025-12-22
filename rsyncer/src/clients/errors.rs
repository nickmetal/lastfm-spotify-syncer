use rspotify::ClientError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse transmit data, error: {0}")]
    ParseError(String),

    // Disconnect(#[from] io::Error),
    #[error("LastFM error: {0}")]
    LastFMError(#[from] lastfm_rust::Error),

    #[error("LastFM Deserialization error: {0}")]
    LastFMDeserializationError(#[from] serde_json::Error),

    #[error("LastFM API unexpected response: {0}")]
    LastFMUnexpectedResponse(String),

    #[error("Spotify error: {0}")]
    SpotifyError(#[from] ClientError),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] async_duckdb::Error),
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
