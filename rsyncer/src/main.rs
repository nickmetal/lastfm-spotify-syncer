use log::info;

mod cli;
mod syncer;

fn init_logger() {
    use env_logger::Env;
    use log::LevelFilter;
    let mut builder = env_logger::Builder::from_env(Env::default());
    // Disable logs from rspotify_http as they are too verbose
    builder.filter_module("rspotify_http", LevelFilter::Off);
    builder.init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load env vars from .env file if ENV_FILE_PATH is set.
    if let Ok(env_file) = std::env::var("ENV_FILE_PATH") {
        dotenv::from_filename(env_file).ok();
        info!("Loaded environment variables from .env file");
    }
    init_logger();
    cli::run().await?;
    Ok(())
}
