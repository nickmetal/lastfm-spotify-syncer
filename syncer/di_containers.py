"""Containers module."""

import logging
import sqlite3
from pydantic import BaseSettings, Field

from dependency_injector import containers, providers


from services.last_fm import LastFmService
from services.spotify import SpotifyService


class LastFMConfig(BaseSettings):
    __prefix = 'LASTFM'

    user: str = Field(env=f'{__prefix}_USER')
    password: str = Field(env=f'{__prefix}_PASSWORD')
    api_key: str = Field(env=f'{__prefix}_API_KEY')
    api_secret: str = Field(env=f'{__prefix}_API_SECRET')

class SpotifyConfig(BaseSettings):
    __prefix = 'SPOTIFY'

    client_id: str = Field(env=f'{__prefix}_CLIENT_ID')
    client_secret: str = Field(env=f'{__prefix}_SECRET')


class Settings(BaseSettings):
    lastfm: LastFMConfig = LastFMConfig()
    spotify: SpotifyConfig = SpotifyConfig()
    logger_name: str = 'app_syncer'


def get_logger(logger_name):
    logging.basicConfig(level=logging.INFO)
    logger = logging.getLogger(logger_name)
    logger.setLevel(logging.INFO)
    return logger


class DIContainer(containers.DeclarativeContainer):

    config = providers.Configuration()

    # Services
    logger = providers.Factory(
        get_logger,
        logger_name=config.logger_name,
    )
    lastfm_service = providers.Singleton(
        LastFmService,
        user=config.lastfm.user,
        password=config.lastfm.password,
        api_key=config.lastfm.api_key,
        api_secret=config.lastfm.api_secret,
        logger=logger,
    )

    spotify_service = providers.Singleton(
        SpotifyService,
        client_id=config.spotify.client_id,
        client_secret=config.spotify.client_secret,
        logger=logger,
    )