use std::sync::Arc;

use clap::{Parser, Subcommand};
use log::info;
use rsyncer_cli::clients::storage::LocalStorage;
use rsyncer_core::{LastFmClient, SpotifyClient, Syncer, errors::Result};

#[derive(Parser)]
#[command(name = "rsyncer")]
#[command(version, about = "Sync liked tracks from Spotify to Last.fm", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start sync process. Note: processed tracks will be cache into local files cache to speed up subsequent runs.
    Sync {},
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Sync {} => {
            sync_tracks().await?;
        }
    }
    Ok(())
}

async fn sync_tracks() -> Result<usize> {
    info!("Building config ...");
    let storage = LocalStorage::try_default().await?;
    let storage = Arc::new(storage);
    storage.init().await?;

    let spotify = Arc::new(SpotifyClient::try_default()?);
    // // CLI prompts may be shown on those two calls
    spotify.authorize_client().await?;

    let mut lastfm = LastFmClient::try_default()?;
    lastfm.authorize_client(storage.as_ref()).await?;
    let lastfm = Arc::new(lastfm);

    let concurrency = 10;

    info!("Authorizing clients ...");

    let syncer = Syncer::new(spotify, lastfm, storage, concurrency);
    syncer.sync().await
}
