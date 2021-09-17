import pylast

from typing import Dict, Generator, Iterator, List, Optional, Tuple


class LastFmService:
    def __init__(self, user, password, api_key, api_secret, logger) -> None:
        logger.info(f'{self} created')
        self.logger = logger 
        self.network = pylast.LastFMNetwork(
            api_key=api_key,
            api_secret=api_secret,
            username=user,
            password_hash=pylast.md5(password),
        )
 
    def get_track(self, artist_name: str, track_name: str) -> pylast.Track:
        track = self.network.get_track(artist=artist_name, title=track_name)
        # check that track exists
        track.is_streamable()
        return track

    def like_track(self, track: pylast.Track):
        track.love()

    def get_liked_tracks(self, limit=50) -> Generator:
        user = self.network.get_authenticated_user()
        tracks = user.get_loved_tracks(limit=limit)
        return tracks