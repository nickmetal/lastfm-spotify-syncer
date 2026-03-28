//! HTTP API server for rsyncer
//!
//! A simple HTTP API that can be deployed to Cloud Run.

use axum::{Json, Router, routing::get};
use log::info;
use serde::Serialize;
use std::net::SocketAddr;

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
    axum::serve(listener, app).await.unwrap();
}
