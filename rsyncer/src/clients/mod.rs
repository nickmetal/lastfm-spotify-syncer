pub mod entities;
pub mod errors;
pub mod lastfm;
pub mod local_storage;
pub mod spotify;

pub use lastfm::LastFmClient;
pub use local_storage::LocalStorage;
pub use spotify::SpotifyClient;
