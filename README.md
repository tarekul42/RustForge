# Skill Workshop Platform — Rust Backend

> Rust update of the [Skill Workshop Management System Backend](https://github.com/tarekul42/skill-workshop-management-system-backend). A production-grade, type-safe rewrite of the original Node.js/TypeScript backend in Rust, preserving all business logic with improved performance, memory safety, and observability.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)

---

## Overview

This is the Rust rewrite of the **Skill Workshop Management System Backend** — a platform that bridges industry experts and students through workshops. The original TypeScript codebase was fully migrated to Rust using a hexagonal architecture (6 crates), trading dynamic dispatch for compile-time guarantees.

The API handles authentication (JWT access + refresh tokens), OTP-verified registration, workshop lifecycle management, SSLCommerz payments with PDF invoice generation, enrollment transactions with snapshot isolation, role-based access control, audit trails, and real-time Prometheus/OTLP observability.

Key differences from the original:
- **~10x faster HTTP throughput** (Axum + Tokio)
- **Zero null-pointer exceptions** (Rust ownership model)
- **Type-safe SQL** (SQLx with compile-time query checking)
- **Native async everywhere** (Tokio, no event-loop blocking)
- **Consolidated observability** (OpenTelemetry + Prometheus metrics + structured tracing)

---

## Tech Stack

| Category | Technology |
|----------|-----------|
| Language | Rust (edition 2024) |
| Web Framework | Axum 0.8 |
| Database | PostgreSQL 16 (SQLx) |
| Object Storage | MinIO / S3 (aws-sdk-s3) |
| Queue | Sidekiq-compatible (redis-rs) |
| Auth | JWT (jsonwebtoken + argon2) |
| Validation | Serde + custom validators |
| Logging | Tracing (tracing-subscriber + OpenTelemetry) |
| PDF | printpdf / lopdf |
| Email | Lettre (SMTP) |
| Payments | SSLCommerz (reqwest) |
| Observability | OpenTelemetry OTLP + Prometheus |
| Containerization | Docker + Docker Compose |
| CI/CD | GitHub Actions (lint, audit, deny, test, build) |

---

## Main Features

- **Hexagonal Architecture** — 6 crates (`shared`, `domain`, `application`, `infrastructure`, `api`, `worker`) with strict dependency layering
- **Multi-modal authentication** — JWT access + refresh token rotation, Google OAuth 2.0, credentials-based login, token blacklisting
- **Automated OTP system** — Redis-backed OTP for account verification and password resets, with TTL expiry
- **SSLCommerz payment integration** — production-ready payment gateway with transaction logging, IPN validation, and PDF invoice generation
- **Snapshot isolation for enrollments** — collision-safe transaction IDs prevent overselling workshops under concurrent enrollment requests
- **Consolidated admin analytics** — single `/api/v1/stats/dashboard` endpoint replaces 7 separate admin calls
- **Audit trail system** — real-time logging of all write operations with entity-level tracking
- **Background jobs** — Redis-backed worker for emails, PDF generation, and async processing
- **OpenTelemetry observability** — traces and metrics exported via OTLP, Prometheus `/metrics` endpoint (API-key protected)
- **Soft deletes** — data recovery and audit-ready deletion mechanism for core models
- **Rate limiting** — configurable per-endpoint rate limits
- **Structured logging** — JSON logging in production, pretty-print in development

---

## Run Locally

### Prerequisites

- Rust 1.75+ (stable toolchain)
- Docker + Docker Compose
- `cargo-audit` (optional: `cargo install cargo-audit`)
- `cargo-deny` (optional: `cargo install cargo-deny`)

### Option A: Docker Compose (recommended)

```bash
# 1. Clone
git clone https://github.com/tarekul42/RustForge.git
cd RustForge

# 2. Configure environment
cp .env.example .env
# Edit .env with your values

# 3. Start infrastructure + API
docker compose up -d
```

This spins up:
- **PostgreSQL 16** on port 5432
- **MinIO** (S3-compatible storage) on ports 9000/9001
- **Mailpit** (SMTP testing) on ports 1025/8025

### Option B: Manual setup

```bash
# 1. Start infrastructure only
docker compose up -d postgres minio mailpit

# 2. Run database migrations
cargo run --bin migration -- up

# 3. Start API server
cargo run
```

Server starts at http://localhost:5000

### Verify it works

```bash
curl http://localhost:5000/api/v1/health
# {"success":true,"data":{"status":"ok"}}
```

### Environment Variables

Copy `.env.example` to `.env` and fill in values. Key variables:

| Variable | Required | Description |
|----------|----------|-------------|
| `APP_DATABASE__URL` | Yes | Postgres connection string |
| `APP_SERVER__PORT` | No | API port (default: 5000) |
| `APP_OBSERVABILITY__LOG_LEVEL` | No | Log level (default: info) |
| `APP_OBSERVABILITY__OTLP_ENDPOINT` | No | OpenTelemetry collector endpoint |
| `APP_OBSERVABILITY__METRICS_PORT` | No | Prometheus metrics port (default: 9000) |
| `METRICS_API_KEY` | No | API key for `/metrics` endpoint |

See `.env.example` for the full list.

---

## Development

| Command | Description |
|---------|-------------|
| `cargo build --all-features` | Build all crates |
| `cargo test --all-features` | Run all tests |
| `cargo clippy --all-targets --all-features -- -D warnings` | Lint |
| `cargo fmt --all -- --check` | Check formatting |
| `cargo audit` | Security audit |
| `cargo deny check` | License/advisory/bans check |

### Project Structure

```
rust-skill-workshop-backend/
├── Cargo.toml              # Workspace manifest
├── docker-compose.yml      # Local dev infrastructure
├── Dockerfile              # Production container (distroless)
├── config/
│   └── default.toml        # Non-secret defaults
├── crates/
│   ├── shared/             # Config, logging, metrics, crypto
│   ├── domain/             # Pure business logic (no I/O)
│   ├── application/        # Use-case orchestration
│   ├── infrastructure/     # Adapters (Postgres, S3, SMTP, etc.)
│   ├── api/                # HTTP server (Axum)
│   └── worker/             # Background job runner
├── migrations/             # SQL migrations
├── templates/              # Email templates
├── benchmarks/             # Load test (k6)
└── vendor/                 # Vendored dependencies (lopdf)
```

---

## License

MIT

---

<div align="center">

**⭐ If this project helped you, give it a star!**

Built by [Tarekul Islam Rifat](https://github.com/tarekul42)

</div>
