# ============================================================
# Stage 1: Chef — prepare dependency recipe for caching
# ============================================================
FROM rust:1.88-bookworm AS chef

RUN cargo install cargo-chef --locked
WORKDIR /app

# ============================================================
# Stage 2: Planner — compute dependency recipe
# ============================================================
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================
# Stage 3: Rust builder — compile dependencies, then the app
# ============================================================
FROM chef AS rust-builder

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --bin rustvault

# ============================================================
# Stage 4: Node builder — build frontend assets
# ============================================================
FROM node:22-alpine AS node-builder

WORKDIR /app/web

COPY web/package.json web/bun.lockb* ./
RUN npm install --frozen-lockfile 2>/dev/null || npm install

COPY web/ .
RUN npm run build

# ============================================================
# Stage 5: Runtime — minimal production image
# ============================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash rustvault

WORKDIR /app

# Copy the compiled binary
COPY --from=rust-builder /app/target/release/rustvault /app/rustvault

# Copy frontend build output
COPY --from=node-builder /app/web/dist /app/static

# Copy default config
COPY docker/config.toml /app/config.toml

USER rustvault

EXPOSE 8080

ENV RUST_LOG=info
ENV RUSTVAULT_CONFIG=/app/config.toml

ENTRYPOINT ["/app/rustvault"]
