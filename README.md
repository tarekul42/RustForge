# Skill Workshop Platform — Rust Backend

A production-grade backend for the Skill Workshop Platform, rebuilt in Rust.

## Quick Start

### Prerequisites

- Rust 1.75+ (stable toolchain)
- Docker + Docker Compose
- `cargo-audit` (optional: `cargo install cargo-audit`)

### 1. Start infrastructure

```bash
docker compose up -d
```

This starts:
- **PostgreSQL 16** on port 5432
- **MinIO** (S3-compatible storage) on ports 9000/9001
- **Mailpit** (SMTP testing) on ports 1025/8025

### 2. Start the API server

```bash
cargo run
```

The server starts on `http://localhost:5000`.

### 3. Verify it works

```bash
curl http://localhost:5000/api/v1/health
# {"success":true,"data":{"status":"ok"}}
```

## Project Structure

```
skill-workshop-rs/
├── Cargo.toml              # Workspace manifest
├── docker-compose.yml      # Local dev infrastructure
├── config/
│   └── default.toml        # Non-secret defaults
├── crates/
│   ├── shared/             # Config, logging, metrics, crypto
│   ├── domain/             # Pure business logic (no I/O)
│   ├── application/        # Use-case orchestration
│   ├── infrastructure/     # Adapters (Postgres, S3, SMTP, etc.)
│   ├── api/                # HTTP server (Axum)
│   └── worker/             # Background job runner (coming in Phase 6)
├── migrations/             # SQL migrations (coming in Phase 1)
└── templates/              # Email templates (coming in Phase 6)
```

## Environment Variables

Copy `.env.example` to `.env` and fill in values. Key variables:

| Variable | Required | Description |
|----------|----------|-------------|
| `APP_DATABASE__URL` | Yes | Postgres connection string |
| `APP_SERVER__PORT` | No | API port (default: 5000) |
| `APP_OBSERVABILITY__LOG_LEVEL` | No | Log level (default: info) |

See `.env.example` for the full list.

## Development

```bash
# Build all crates
cargo build --all-features

# Run tests
cargo test --all-features

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all -- --check
```

## License

MIT
