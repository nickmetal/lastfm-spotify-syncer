/// Data entities for tracks and artists
pub mod entities;
/// Error types and result aliases
pub mod errors;
/// Last.fm API client
pub mod lastfm;
/// Local storage using `DuckDB`
pub mod local_storage;
/// Spotify API client
pub mod spotify;

pub use lastfm::LastFmClient;
pub use local_storage::LocalStorage;
pub use spotify::SpotifyClient;
