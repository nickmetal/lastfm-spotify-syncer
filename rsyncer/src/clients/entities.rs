#[derive(Debug)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artist: Artist, // assume one artist for simplicity
    pub url: String,
}
