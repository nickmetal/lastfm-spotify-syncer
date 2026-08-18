//! HTTP API server for rsyncer
//!
//! A simple HTTP API that can be deployed to Cloud Run.
use axum::routing::post;
use axum::{Router, routing::get};
use log::info;
use rsyncer_api::{
    APILastFMStorage, health_handler, init_app_state, init_logger, process_shutdown_signal,
    sync_tracks_handler,
};
use rsyncer_core::errors::Result;
use rsyncer_core::{LastFmClient, SpotifyClient};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let env_file = std::env::var("ENV_FILE_PATH").unwrap_or_else(|_| ".env".to_string());
    dotenvy::from_path(env_file).ok();

    // Initialize logger
    init_logger();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let migrations_path =
        PathBuf::from(std::env::var("MIGRATIONS_PATH").expect("MIGRATIONS_PATH not set"));

    let mut storage = APILastFMStorage::new(database_url, migrations_path);
    storage.connect().await?;
    let storage = Arc::new(storage);

    let spotify = Arc::new(SpotifyClient::try_default()?);
    let lastfm = LastFmClient::try_default()?;
    let app_state = init_app_state(storage, spotify, lastfm).await?;
    let app_state = Arc::new(app_state);

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/tracks/sync", post(sync_tracks_handler).with_state(app_state));

    // Get port from environment (Cloud Run sets PORT)
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting HTTP API server on {}:{}", addr, port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Server ready. Waiting for shutdown signal (SIGINT/SIGTERM)...");
    axum::serve(listener, app).with_graceful_shutdown(process_shutdown_signal()).await.unwrap();
    info!("Server has shut down gracefully.");
    Ok(())
}
