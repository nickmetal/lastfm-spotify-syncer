use clap::{Parser, Subcommand};
use log::info;
use rsyncer::clients::errors::Result;

use crate::syncer;

#[derive(Parser)]
#[command(name = "rsyncer")]
#[command(version, about = "Sync liked tracks from Spotify to Last.fm", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

async fn sync_tracks() -> Result<()> {
    info!("Building config ...");
    let mut config = syncer::ConfigBuilder::new().build().await?;
    info!("Authorizing clients ...");
    config.storage.init_db().await?;
    // CLI prompts may be shown on those two calls
    config.spotify.authorize_client().await?;
    // Some of the LastFM methods(3d party crate) may panic if not authorized
    config.lastfm.authorize_client().await?;
    let syncer = syncer::Syncer::new(config);
    syncer.sync().await
}
