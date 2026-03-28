/// Data entities for tracks and artists
pub mod entities;
/// Error types and result aliases
pub mod errors;
/// Last.fm API client
#[cfg(feature = "cli")]
pub mod lastfm;
/// Local storage using `DuckDB`
#[cfg(feature = "cli")]
pub mod local_storage;
/// Spotify API client
#[cfg(feature = "cli")]
pub mod spotify;

#[cfg(feature = "cli")]
pub use lastfm::LastFmClient;
#[cfg(feature = "cli")]
pub use local_storage::LocalStorage;
#[cfg(feature = "cli")]
pub use spotify::SpotifyClient;
