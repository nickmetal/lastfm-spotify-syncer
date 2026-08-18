/// Represents a music artist
#[derive(Debug, Clone)]
pub struct Artist {
    /// The name of the artist
    pub name: String,
}

/// Represents a music track with associated artist
#[derive(Debug, Clone)]
pub struct Track {
    /// Unique identifier for the track (typically from Spotify)
    pub id: String,
    /// The name/title of the track
    pub name: String,
    /// The artist who created the track (simplified to one artist)
    pub artist: Artist,
}
