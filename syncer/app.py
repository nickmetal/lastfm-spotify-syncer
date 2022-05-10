import json
import pickle
import logging
import time
import os
from typing import Dict, Generator, Iterator, List, Optional, Tuple

import pylast

# todo: remove
# import rumps  # mac os ui lib

from fuzzywuzzy import fuzz

from dependency_injector.wiring import inject, Provide
from services.last_fm import LastFmService
from services.spotify import SpotifyService
from syncer.di_containers import DIContainer, Settings
from syncer.model import SyncTrack


class Syncer:
    @inject
    def __init__(self, 
                 lastfm_service: LastFmService = Provide[DIContainer.lastfm_service],
                 spotify_service: SpotifyService = Provide[DIContainer.spotify_service],
                 logger: logging.Logger = Provide[DIContainer.logger],
                ):
        self.lastfm_service = lastfm_service
        self.spotify_service = spotify_service
        self.logger = logger 
        self._cache_file = '.cache_processed'

    def _load_processed_tracks(self) -> List[str]:
        self.logger.info(f'download syncer cache: {self._cache_file}')

        with open(self._cache_file, 'r+') as f:
            saved_track_ids = f.read().strip() or '[]'
            return json.loads(saved_track_ids)

    def _dump_processed_tracks(self, track_ids: List[str]):
        self.logger.info(f'store syncer cache: {self._cache_file}')
        with open(self._cache_file, 'w') as f:
            return json.dump(track_ids, f)

    def sync_spotify_likes_with_lastfm(self):
        cached_track_ids = set(self._load_processed_tracks())
        new_ids = set()

        for track in self.spotify_service.get_liked_tracks():
            track_id = track['id']
            if track_id in cached_track_ids:
                self.logger.debug(f'skip cached/processed track: {track}')
                continue

            lastfm_track = self.lastfm_service.get_track(track['artist'], track['name'])
            self.logger.info(f'set like for track: {track}')

            self.lastfm_service.like_track(lastfm_track)
            new_ids.add(track_id)

        if new_ids.difference(cached_track_ids):
            new_cache = cached_track_ids.union(new_ids)
            self._dump_processed_tracks(list(new_cache))
        else:
            self.logger.info('all Spotify tracks already synced with LastFM')

    def sync_liked_tracks_from_lastfm_with_spotify(self):
        """Gets tracks from last fm liked list and searches them in Spotifyself.

            Method sometimes requires user input from STDIN. 
        """
        tracks_to_load = []
        missed_tracks = []

        playlist = self.spotify_service.get_custom_liked_playlist()
        if not playlist:
            raise Exception('Spotify doesnt have custom playlist for adding lastfm likes')

        playlist_tracks = self.spotify_service.spotify.playlist_items(playlist_id=playlist['id'])['items']
        track_uris = [track['track']['uri'] for track in playlist_tracks]

        for track in self.lastfm_service.get_liked_tracks(limit=None):
            query = f'{track.track.artist.name} {track.track.get_name()}'
            results = self.spotify_service.spotify.search(q=query, type='track', limit=50)

            sync_track = self._find_search_match(track, results)
            if sync_track and sync_track.spotify_track_uri not in track_uris:
                self.logger.info(f'found match: {track}')
                tracks_to_load.append(sync_track)
            else:
                self.logger.info(f'no info for: {track} in spotify')
                missed_tracks.append(track)

        self.logger.info(f'load liked tracks to spotify: {len(tracks_to_load)}')
        if tracks_to_load:
            self._add_liked_tracks_to_spotify(tracks_to_load, playlist)

        self.logger.info(f'store missed tracks: {len(missed_tracks)}')
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
                        self.logger.info('answer is "no"')
                
    def _add_liked_tracks_to_spotify(self, tracks_to_load: List[SyncTrack], playlist: Dict):
        from itertools import islice

        def do_chunk(it, size):
            it = iter(it)
            return iter(lambda: tuple(islice(it, size)), ())

        for chunk in do_chunk(tracks_to_load, 100):
            track_ids = [track.spotify_track_uri for track in chunk]
            self.spotify_service.spotify.playlist_add_items(playlist_id=playlist['id'], items=track_ids)

    def _store_missed_liked_tracks(self, missed_tracks):
        with open('.missed_spotify_tracks', 'wb') as file:
            pickle.dump(missed_tracks, file)


if __name__ == "__main__":
    import sys
    container = DIContainer()
    container.init_resources()
    container.config.from_pydantic(Settings())
    container.wire(modules=[sys.modules[__name__]])

    Syncer().sync_spotify_likes_with_lastfm()
    # Syncer().sync_liked_tracks_from_lastfm_with_spotify()
    
    container.shutdown_resources()
