Rsyncer - a rust implementation of Track likes syncer. That is my Pet project for learning Rust and building a simple HTTP API with Cloud Run.

### Warning
> This tool relies on "lastfm-rust" library which is not not production ready: it prints debug information to stdout and doesn't handle errors properly. Use at your own risk.



### Envs vars

```
# As env variables:
export RSPOTIFY_CLIENT_ID=your_spotify_client_id
export RSPOTIFY_CLIENT_SECRET=your_spotify_client_secret
export RSPOTIFY_REDIRECT_URI=http://127.0.0.1:8888/callback # example for local development
export LASTFM_API_KEY=your_lastfm_api_key
export LASTFM_API_SECRET=your_lastfm_api_secret
export RUST_LOG=info

# or in a .env file:
export ENV_FILE_PATH=.path/to/your/.env
```


### CLI

```sh
# run syncer as a CLI tool
just run-dev-cli

```sh
...
**Note:** The HTTP API is built without DuckDB and other CLI-only dependencies. This results in faster builds and smaller binary size.
cargo test --features cli
# with coverage
cargo llvm-cov --summary-only --features cli
```

### Code formatting and linting
```sh
just lint
just test
```


### Bump version

```sh
# cargo install cargo-edit
cargo set-version --bump minor
```


# HTTP API

The project includes a simple HTTP API (`http_api`) for deployment to Cloud Run.

**Note:** The HTTP API is built without the `cli` feature, which excludes DuckDB and other CLI-only dependencies. This results in faster builds and smaller binary size.

### Authendication
The API uses GCP IAM for authentication from TF `cloud_run_allowed_users`.
Access example:
```sh
curl -H "Authorization: Bearer $(gcloud auth print-identity-token)" https://YOUR_CLOUD_RUN_URL/your-endpoint
```

### DB management
The API uses `sqlx` for database management. The database is managed via migrations located in `rsyncer_api/migrations`. To run migrations, use the following command:
```sh
cargo sqlx migrate run --source rsyncer_api/migrations --database-url=postgresql://postgres:postgres@localhost:5432/rsyncer
```


### Endpoints
- `GET /` - Hello world
- `GET /health` - Health check with version info

### Local Development
```sh
cargo run --bin http_api
# Then: curl http://localhost:8080/health
```

## Docker Build

### Local Docker Build
```sh
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
docker build --build-arg VERSION=$VERSION -t rsyncer-api:$VERSION .

# Run locally
docker run -p 8080:8080 rsyncer-api:$VERSION
```

## Cloud Deployment

**Prerequisites:** Run `terraform apply` first to create the Artifact Registry repository and enable required APIs.

### Build with Cloud Build
```sh
gcloud builds submit --config=cloudbuild.yaml .
```
