mod cli;
mod syncer;
use env_logger::Env;
use log::LevelFilter;
use rsyncer::clients::errors::Result;

fn init_logger() {
    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("info"));
    // Disable logs from rspotify_http as they are too verbose
    builder.filter_module("rspotify_http", LevelFilter::Off);
    builder.init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load env vars from ENV_FILE_PATH if set, otherwise from .env file if present.
    if let Ok(env_path) = std::env::var("ENV_FILE_PATH") {
        dotenvy::from_path(env_path).ok();
    } else {
        dotenvy::from_filename(".env").ok();
    }
    init_logger();
    cli::run().await
}
