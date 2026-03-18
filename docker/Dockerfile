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
COPY locales/ locales/
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================
# Stage 3: Rust builder — compile dependencies, then the app
# ============================================================
FROM chef AS rust-builder

# Build profile: "release" (default) or "debug"
ARG PROFILE=release

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (cached layer — must match the profile used below)
RUN if [ "$PROFILE" = "release" ]; then \
        cargo chef cook --release --recipe-path recipe.json; \
    else \
        cargo chef cook --recipe-path recipe.json; \
    fi

# Build application
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY locales/ locales/
RUN if [ "$PROFILE" = "release" ]; then \
        cargo build --release --bin rustvault; \
    else \
        cargo build --bin rustvault; \
    fi

# ============================================================
# Stage 4: Bun builder — build frontend assets
# ============================================================
FROM oven/bun:1 AS node-builder

WORKDIR /app/web

COPY web/package.json web/bun.lock ./
RUN bun install --frozen-lockfile

COPY web/ .
RUN bun run build

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
# ARG must be re-declared after FROM to be visible in this stage
ARG PROFILE=release
COPY --from=rust-builder /app/target/${PROFILE}/rustvault /app/rustvault

# Copy frontend build output
COPY --from=node-builder /app/web/dist /app/static

# Copy locale bundles
COPY locales/ /app/locales/

# Copy default config
COPY docker/config.toml /app/config.toml

USER rustvault

EXPOSE 8080

ENV RUST_LOG=info
ENV RUSTVAULT_CONFIG=/app/config.toml

ENTRYPOINT ["/app/rustvault"]
