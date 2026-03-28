//! HTTP API server for rsyncer
//!
//! A simple HTTP API that can be deployed to Cloud Run.

use axum::{Json, Router, routing::get};
use log::{info, warn};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::signal;

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// Hello response
#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

/// Health check endpoint
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok", version: env!("CARGO_PKG_VERSION") })
}

/// Hello world endpoint
async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse { message: "Hello from rsyncer API!".to_string() })
}

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Build router
    let app = Router::new().route("/", get(hello)).route("/health", get(health));

    // Get port from environment (Cloud Run sets PORT)
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting HTTP API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Server ready. Waiting for shutdown signal (SIGINT/SIGTERM)...");
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await.unwrap();
    info!("Server has shut down gracefully.");
}

/// Wait for SIGINT or SIGTERM and log shutdown event
async fn shutdown_signal() {
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
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
