# syntax=docker/dockerfile:1

# Build arguments
ARG VERSION=0.0.0
ARG GIT_SHA=unknown

# =============================================================================
# Stage 1: Build the Rust binary
# =============================================================================

FROM rust:1.92-slim-trixie AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Leverage Docker cache: copy manifests first, build dependencies, then source
COPY Cargo.toml Cargo.lock ./
RUN mkdir src
COPY src/main.rs src/
RUN cargo fetch
RUN cargo build --release --bin http_api || true

# Now copy the rest of the source (invalidates cache only if code changes)
COPY src ./src


# Final build
RUN cargo build --release --bin http_api

# Strip debug symbols to reduce binary size
RUN strip target/release/http_api

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
COPY --from=builder /app/target/release/http_api /http_api

# Cloud Run uses PORT environment variable
ENV PORT=8080

# Expose the port
EXPOSE 8080

# Run as non-root user (distroless:nonroot runs as uid 65532)
USER nonroot

# Run the binary
ENTRYPOINT ["/http_api"]
