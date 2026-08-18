/// Data entities for tracks and artists
pub mod entities;
/// Error types and result aliases
pub mod errors;
/// Last.fm API client
pub mod lastfm;
/// Spotify API client
pub mod spotify;
/// Local storage using `DuckDB`
pub mod storage;

pub use lastfm::LastFmClient;

pub use spotify::SpotifyClient;
