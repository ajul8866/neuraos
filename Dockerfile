# ============================================================
# NeuraOS — Multi-stage Docker build
# Stage 1: chef     (cargo-chef dependency caching)
# Stage 2: builder  (rust:1.80-slim-bookworm — full compile)
# Stage 3: runtime  (debian:bookworm-slim — minimal image)
# ============================================================

# ── Stage 1: Dependency planner (cargo-chef) ─────────────────
FROM rust:1.80-slim-bookworm AS chef

RUN cargo install cargo-chef --locked
WORKDIR /app

# ── Stage 2: Dependency caching layer ────────────────────────
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
# Generate the recipe file — only changes when Cargo.toml files change
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: Builder ─────────────────────────────────────────
FROM chef AS builder

# Install system build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config        \
    libssl-dev        \
    libsqlite3-dev    \
    curl              \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cook dependencies first — this layer is cached unless deps change
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy full source and build the release binary
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --bin neuraos

# ── Stage 4: Runtime ─────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.title="NeuraOS"
LABEL org.opencontainers.image.description="The Agent Operating System — autonomous AI agents"
LABEL org.opencontainers.image.source="https://github.com/neuraos/neuraos"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"

WORKDIR /app

# Install minimal runtime dependencies (no build tools)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates  \
    libssl3           \
    libsqlite3-0      \
    curl              \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN groupadd --gid 1001 neuraos \
    && useradd  \
        --uid 1001 \
        --gid neuraos \
        --shell /sbin/nologin \
        --no-create-home \
        neuraos

# Create persistent data directories
RUN mkdir -p \
    /var/lib/neuraos/data   \
    /var/lib/neuraos/logs   \
    /var/lib/neuraos/plugins \
    /etc/neuraos             \
    && chown -R neuraos:neuraos \
        /var/lib/neuraos        \
        /etc/neuraos

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/neuraos /usr/local/bin/neuraos
RUN chmod +x /usr/local/bin/neuraos

# Copy default configuration files
COPY config/ /etc/neuraos/

# Volumes for persistence
VOLUME ["/var/lib/neuraos/data", "/var/lib/neuraos/logs"]

# Switch to non-root user
USER neuraos

# Expose HTTP API port
EXPOSE 8080

# Health check — polls /health every 30 seconds
HEALTHCHECK \
    --interval=30s  \
    --timeout=10s   \
    --start-period=15s \
    --retries=3     \
    CMD curl -sf http://localhost:8080/health | grep -q '"status":"ok"' || exit 1

# Default startup command
ENTRYPOINT ["/usr/local/bin/neuraos"]
CMD ["--config", "/etc/neuraos/default.toml", "--log-level", "info"]
