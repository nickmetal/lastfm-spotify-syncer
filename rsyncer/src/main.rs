mod syncer;
use log::info;
use rsyncer::clients::LocalStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut config = syncer::ConfigBuilder::new().build().await?;

    info!("Authorizing clients ...");
    config.storage.init_db().await?;
    // CLI prompts may be shown on those two calls
    config.spotify.authorize_client().await?;
    // Some of the LastFM methods(3d party crate) may panic if not authorized
    config.lastfm.authorize_client().await?;

    let syncer = syncer::Syncer::new(config);
    syncer.sync().await?;

    Ok(())
}
