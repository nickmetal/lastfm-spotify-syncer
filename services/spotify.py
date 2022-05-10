import logging
import time

import spotipy
from spotipy.oauth2 import SpotifyOAuth
from typing import Dict, Generator, Iterator, List, Optional, Tuple


class SpotifyService:
    def __init__(self, client_id, client_secret, logger: logging.Logger) -> None:
        self.logger = logger
        self.spotify = spotipy.Spotify(auth_manager=SpotifyOAuth(client_id=client_id,
                                                                 client_secret=client_secret,
                                                                 show_dialog=True,
                                                                 redirect_uri="http://localhost:8888/callback",
                                                                 scope="user-read-email user-library-read"))

    def get_liked_tracks(self) -> Generator[Dict, None, None]:
        # https://developer.spotify.com/documentation/web-api/reference/endpoint-get-users-saved-tracks
        all_fetched = False
        tracks = []
        fetched_count = 0
        limit = 50
        offset = 0

        try:
            while not all_fetched:
                time.sleep(0.2)
                response = self.spotify.current_user_saved_tracks(limit=limit, offset=offset)
                new_tracks = response['items']

                fetched_count += len(new_tracks) 
                all_fetched = fetched_count == response['total']
                offset += len(new_tracks)

                self.logger.info(f'fetched {fetched_count}/{response["total"]} from Spotify API')

                for track in new_tracks:
                    yield {
                        'artist': track['track']['artists'][0]['name'],
                        'name': track['track']['name'],
                        'id':track['track']['id'],
                    }
        except spotipy.oauth2.SpotifyOauthError as e:
            raise Exception(f'{e}. Try to remove auth cache. Run rm .cache in current dir')

    def get_custom_liked_playlist(self, playlist_name: str = 'lastfm_liked') -> Optional[Dict]:
        
        for playlist in self.spotify.current_user_playlists()['items']:
            if playlist['name'] == playlist_name:
                return playlist