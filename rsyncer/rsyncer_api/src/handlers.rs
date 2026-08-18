use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use log::{error, info};
use serde::Serialize;

use crate::AppState;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

/// Hello response
#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// Health check endpoint
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok", version: env!("CARGO_PKG_VERSION") })
}

/// sync-tracks endpoint
pub async fn sync_tracks_handler(
    State(app_state): State<Arc<AppState>>,
) -> (StatusCode, Json<MessageResponse>) {
    info!("Sync handler");
    let syncer = app_state.syncer.as_ref();

    match syncer.sync().await {
        Ok(synced_count) => (
            StatusCode::OK,
            Json(MessageResponse {
                message: format!("Synced {} tracks successfully.", synced_count),
            }),
        ),
        Err(error) => {
            error!("Sync failed: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse { message: "500".to_string() }),
            )
        }
    }
}
