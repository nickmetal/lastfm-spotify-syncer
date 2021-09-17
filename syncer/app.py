import json
import pickle
import logging
import time
import os
from typing import Dict, Generator, Iterator, List, Optional, Tuple

import pylast

import rumps
import spotipy
from spotipy.oauth2 import SpotifyOAuth
from fuzzywuzzy import fuzz

from syncer.model import SyncTrack


logger = logging.getLogger(__name__)
# logging.basicConfig(format='%(asctime)-15s %(clientip)s %(user)-8s %(message)s')
logger.addHandler(logging.StreamHandler())
logger.setLevel(logging.INFO)


# class Playlist:
#     track: str


class LastFmService:
    def __init__(self, user, password, api_key, api_secret) -> None:

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


class SpotifyService:
    def __init__(self, client_id, client_secret) -> None:
        self.spotify = spotipy.Spotify(auth_manager=SpotifyOAuth(client_id=client_id,
                                                                 client_secret=client_secret,
                                                                 show_dialog=True,
                                                                 redirect_uri="http://localhost:8888/callback",
                                                                 scope="playlist-read-private playlist-modify-private user-read-private user-read-email user-library-read"))

    def get_liked_tracks(self) -> Iterator[Dict]:
        # https://developer.spotify.com/documentation/web-api/reference/endpoint-get-users-saved-tracks
        all_fetched = False
        tracks = []
        fetched_count = 0
        limit = 50
        offset = 0

        while not all_fetched:
            time.sleep(0.2)
            response = self.spotify.current_user_saved_tracks(limit=limit, offset=offset)
            new_tracks = response['items']

            fetched_count += len(new_tracks) 
            all_fetched = fetched_count == response['total']
            offset += len(new_tracks)

            logger.info(f'fetched {fetched_count}/{response["total"]} from Spotify API')

            for track in new_tracks:
                yield {
                    'artist': track['track']['artists'][0]['name'],
                    'name': track['track']['name'],
                    'id':track['track']['id'],
                }

    def get_custom_liked_playlist(self, playlist_name: str = 'lastfm_liked') -> Optional[Dict]:
        
        for playlist in self.spotify.current_user_playlists()['items']:
            if playlist['name'] == playlist_name:
                return playlist


class Syncer:
    def __init__(self) -> None:
        user, password = os.environ['LASTFM_CREDS'].split(':')
        self.lastfm_service = LastFmService(
            user, password, api_key=os.environ['LASTFM_API_KEY'], 
            api_secret=os.environ['LASTFM_API_SECRET'],
        )
        self.spotify_service = SpotifyService(
            client_id=os.environ['SPOTIFY_CLIENT_ID'],
            client_secret=os.environ['SPOTIFY_SECRET'],
        )
        self._cache_file = '.cache_processed'

    def sync_lastfm_likes_with_spotify(self):
        raise NotImplemented

    def _load_processed_tracks(self) -> List[str]:
        logger.info(f'download syncer cache: {self._cache_file}')

        with open(self._cache_file, 'r+') as f:
            saved_track_ids = f.read().strip() or '[]'
            return json.loads(saved_track_ids)

    def _dump_processed_tracks(self, track_ids: List[str]):
        logger.info(f'store syncer cache: {self._cache_file}')
        with open(self._cache_file, 'w') as f:
            return json.dump(track_ids, f)

    def sync_spotify_likes_with_lastfm(self):
        cached_track_ids = set(self._load_processed_tracks())
        new_ids = set()

        for track in self.spotify_service.get_liked_tracks():
            track_id = track['id']
            if track_id in cached_track_ids:
                logger.debug(f'skip cached/processed track: {track}')
                continue

            lastfm_track = self.lastfm_service.get_track(track['artist'], track['name'])
            logger.info(f'set like for track: {track}')

            self.lastfm_service.like_track(lastfm_track)
            new_ids.add(track_id)

        if new_ids.difference(cached_track_ids):
            new_cache = cached_track_ids.union(new_ids)
            self._dump_processed_tracks(list(new_cache))
        else:
            logger.info('all Spotify tracks already synced with LastFM')

    def sync_liked_tracks_from_lastfm_with_spotify(self):
        def load_cache():
            with open('lastfm.cache', 'rb') as file:
                return pickle.load(file)

        def _track_exists(sync_track: SyncTrack, tracks: List[Dict]):
            return tracks and any(sync_track.spotify_track_uri == track['uri'] for track in tracks)

        tracks_to_load = []
        missed_tracks = []

        playlist = self.spotify_service.get_custom_liked_playlist()
        if not playlist:
            raise Exception('Spotify doesnt have custom playlist for adding lastfm likes')

        playlist_tracks = self.spotify_service.spotify.playlist_items(playlist_id=playlist['id'])['items']
        track_uris = [track['track']['uri'] for track in playlist_tracks]

        # import ipdb; ipdb.set_trace()
        
        # all_ = list(self.lastfm_service.get_liked_tracks(limit=None))]
        # import ipdb; ipdb.set_trace()

        # for track in self.lastfm_service.get_liked_tracks(limit=None):
        for track in load_cache():
            query = f'{track.track.artist.name} {track.track.get_name()}'
            results = self.spotify_service.spotify.search(q=query, type='track', limit=50)

            sync_track = self._find_search_match(track, results)
            if sync_track and sync_track.spotify_track_uri not in track_uris:
                logger.info(f'found match: {track}')
                tracks_to_load.append(sync_track)
            else:
                logger.info(f'no info for: {track} in spotify')
                missed_tracks.append(track)

        logger.info(f'load liked tracks to spotify: {len(tracks_to_load)}')
        if tracks_to_load:
            self._add_liked_tracks_to_spotify(tracks_to_load, playlist)

        logger.info(f'store missed tracks: {len(missed_tracks)}')
        if missed_tracks:
            self._store_missed_liked_tracks(missed_tracks)

    def _find_search_match(self, track: pylast.LovedTrack, search_results, match_ratio: int = 85) -> Optional[SyncTrack]:
        
        if not search_results['tracks']['total']:
            return
        
        l = lambda s: s.lower()
        def are_tracks_the_same(spotify_pair: Tuple, lastfm_pair: Tuple) -> bool:
            message = f'Are next tracks are the same?: {spotify_pair} and {lastfm_pair}. Answer "y" if yes: '
            answer = input(message).lower().strip()
            return answer == 'y'

        lf_artist, lf_song_name = l(track.track.artist.name), l(track.track.get_name())

        for search_item in search_results['tracks']['items']:
            for artist in search_item['artists']:
                spotify_artist, spotify_song_name = l(artist['name']), l(search_item['name'])
                sync_track = SyncTrack(
                    last_fm_artist=lf_artist,
                    last_fm_song=lf_song_name,
                    spotify_track_uri=search_item['uri'],
                )
                
                artist_ratio = fuzz.ratio(lf_artist, spotify_artist)
                song_ratio = fuzz.ratio(lf_song_name, spotify_song_name)

                if song_ratio >= match_ratio and artist_ratio >= match_ratio:
                    return sync_track

                # (spotify_artist, spotify_song_name),  (lf_artist, lf_song_name), artist_ratio, song_ratio
                if song_ratio + artist_ratio >= 140:
                    yes = are_tracks_the_same((spotify_artist, spotify_song_name), (lf_artist, lf_song_name))
                    if yes:
                        return sync_track
                    else:
                        logger.info('answer is "no"')
                
    def _add_liked_tracks_to_spotify(self, tracks_to_load: List[SyncTrack], playlist: Dict):
        from itertools import islice

        def do_chunk(it, size):
            it = iter(it)
            return iter(lambda: tuple(islice(it, size)), ())

        for chunk in do_chunk(tracks_to_load, 100):
            track_ids = [track.spotify_track_uri for track in chunk]
            self.spotify_service.spotify.playlist_add_items(playlist_id=playlist['id'], items=track_ids)

    def _store_missed_liked_tracks(self, missed_tracks):
        try:
            with open('.missed_spotify_tracks', 'wb') as file:
                pickle.dump(missed_tracks, file)
        except Exception:
            import ipdb; ipdb.set_trace()
            import ipdb; ipdb.set_trace()

            
class AwesomeStatusBarApp(rumps.App):

    # @rumps.clicked("My_exit")
    # def exit(self, _):
    # rumps.quit_application()

    # @rumps.clicked("Silly button")
    # def onoff(self, sender):
    #     sender.state = not sender.state

    @rumps.clicked("notification")
    def sayhi(self, _):
        rumps.notification("Awesome title", "amazing subtitle", "hi!!1")


if __name__ == "__main__":
    # app = AwesomeStatusBarApp("Awesome App")
    # rumps.notification("Lastfm Spotiy Sync", '', 'Started')
    Syncer().sync_spotify_likes_with_lastfm()
    # Syncer().sync_liked_tracks_from_lastfm_with_spotify()
    # rumps.notification("Lastfm Spotiy Sync", '', 'Completed')
    # app.run()
