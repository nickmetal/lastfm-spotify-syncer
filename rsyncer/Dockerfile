# syntax=docker/dockerfile:1

# Build arguments
ARG VERSION=0.0.0
ARG GIT_SHA=unknown

# =============================================================================
# Stage 1: Build the Rust binary
# =============================================================================

FROM rust:1.97-slim-trixie AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    just \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Leverage Docker cache: copy manifests first, build dependencies, then source
COPY Cargo.toml Cargo.lock justfile ./
COPY rsyncer_api/Cargo.toml ./rsyncer_api/Cargo.toml
COPY rsyncer_core/Cargo.toml ./rsyncer_core/Cargo.toml
COPY rsyncer_cli/Cargo.toml ./rsyncer_cli/Cargo.toml

RUN mkdir -p rsyncer_api/src
RUN mkdir -p rsyncer_core/src
RUN mkdir -p rsyncer_cli/src

COPY rsyncer_api/src/main.rs rsyncer_api/src/
COPY rsyncer_core/src/lib.rs rsyncer_core/src/
COPY rsyncer_cli/src/main.rs rsyncer_cli/src/

RUN cargo fetch
RUN just build-bin-api || true

# Now copy the rest of the source (invalidates cache only if code changes)
COPY rsyncer_api/src rsyncer_api/src
COPY rsyncer_core/src rsyncer_core/src
COPY .sqlx .sqlx


# Final build
RUN just build-bin-api
# Copy migrations that are required at runtime
COPY ./rsyncer_api/migrations rsyncer_api/migrations

# =============================================================================
# Stage 2: Create minimal runtime image
# =============================================================================
FROM gcr.io/distroless/cc-debian13:nonroot

# Re-declare ARGs for this stage
ARG VERSION
ARG GIT_SHA

# OCI Image Labels
LABEL org.opencontainers.image.title="rsyncer-api" \
      org.opencontainers.image.description="HTTP API for rsyncer - Spotify to Last.fm sync" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${GIT_SHA}" \
      org.opencontainers.image.source="https://github.com/nickmetal/lastfm-spotify-syncer" \
      org.opencontainers.image.licenses="MIT"

# Copy the binary from builder
COPY --from=builder /app/target/release/rsyncer_api /rsyncer_api
COPY --from=builder /app/rsyncer_api/migrations /migrations

# Run as non-root user (distroless:nonroot runs as uid 65532)
USER nonroot

# Run the binary
ENTRYPOINT ["/rsyncer_api"]
