use log::{debug, info};
use rsyncer::clients::errors::Result;

use crate::syncer;

pub async fn run() -> Result<()> {
    let cmd =
        clap::Command::new("rsyncer").bin_name("rsyncer").subcommand_required(true).subcommand(
            clap::Command::new("sync").about("Synchronize liked tracks between Spotify and LastFM"),
        );
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("sync", _matches)) => sync_tracks().await?,
        _ => unreachable!("clap should ensure we don't get here"),
    };
    Ok(())
}

async fn sync_tracks() -> Result<()> {
    debug!("Building config ...");
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
