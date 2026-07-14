# syntax=docker/dockerfile:1
# Multi-stage production build for Skill Workshop Platform.

# Stage 1: Planner (cargo-chef)
FROM rust:1.81-alpine3.20 AS chef
RUN apk add --no-cache musl-dev && \
    cargo install cargo-chef --version 0.1.70
WORKDIR /app

# Stage 2: Prepare recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (layer caching)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY vendor /app/vendor
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# Stage 4: Build application
FROM builder AS builder-final
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin sw-api --bin sw-worker && \
    cp /app/target/release/sw-api /app/sw-api && \
    cp /app/target/release/sw-worker /app/sw-worker && \
    cp -r /app/templates /app/templates && \
    cp -r /app/migrations /app/migrations && \
    cp /app/config/default.toml /app/config/default.toml

# Stage 5: Runtime (distroless)
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
WORKDIR /app
COPY --from=builder-final --chown=nonroot:nonroot /app/sw-api /app/sw-api
COPY --from=builder-final --chown=nonroot:nonroot /app/sw-worker /app/sw-worker
COPY --from=builder-final --chown=nonroot:nonroot /app/templates /app/templates
COPY --from=builder-final --chown=nonroot:nonroot /app/migrations /app/migrations
COPY --from=builder-final --chown=nonroot:nonroot /app/config /app/config

EXPOSE 5000
EXPOSE 5001

HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
    CMD ["/app/sw-api", "--health-check"]

ENTRYPOINT ["/app/sw-api"]
