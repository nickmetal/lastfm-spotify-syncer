from pydantic import BaseModel


class SyncTrack(BaseModel):
    last_fm_artist: str
    last_fm_song: str

    # spotify_artist: str
    # spotify_song: str

    spotify_track_uri: str
