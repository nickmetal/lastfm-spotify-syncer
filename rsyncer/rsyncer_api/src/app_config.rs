use std::sync::Arc;

use env_logger::Env;
use log::{LevelFilter, info, warn};
use rsyncer_core::{LastFmClient, SpotifyClient, Syncer, errors::Result, storage::LastFMStorage};
use tokio::signal;

// #[derive(Clone)]
pub struct AppState {
    pub syncer: Arc<Syncer>,
}

pub fn init_logger() {
    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("info"));
    // Disable logs from rspotify_http as they are too verbose
    builder.filter_module("rspotify_http", LevelFilter::Off);
    builder.init();
}

pub async fn init_app_state(
    storage: Arc<dyn LastFMStorage>,
    spotify: Arc<SpotifyClient>,
    mut lastfm: LastFmClient,
) -> Result<AppState> {
    info!("Init app state...");
    // // CLI prompts may be shown on those two calls
    spotify.authorize_client().await?;
    lastfm.authorize_client(storage.as_ref()).await?;
    let lastfm = Arc::new(lastfm);
    let concurrency = 10;
    let syncer = Syncer::new(spotify, lastfm, storage, concurrency);
    let syncer = Arc::new(syncer);
    let state = AppState { syncer };
    Ok(state)
}

/// Wait for SIGINT or SIGTERM and log shutdown event
pub async fn process_shutdown_signal() {
    // SIGINT (Ctrl+C)
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        warn!("Received SIGINT (Ctrl+C), shutting down...");
    };
    // SIGTERM (docker stop, podman stop, Cloud Run termination)
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
        warn!("Received SIGTERM, shutting down...");
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
