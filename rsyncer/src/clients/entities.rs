#[derive(Debug, Clone)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artist: Artist, // assume one artist for simplicity
}
