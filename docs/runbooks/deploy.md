# Deployment Runbook

## Overview

Production deployment to Railway/Fly.io. This runbook assumes:
- Git repo is clean (no uncommitted changes)
- CI passes on `main`
- Docker image is built and pushed
- Database migrations are forward-compatible

## Prerequisites

- [ ] `cargo audit` passes (no vulnerabilities)
- [ ] `cargo deny check` passes
- [ ] CI is green on `main`
- [ ] Database migrations are reviewed and tested
- [ ] `.env` contains production secrets (not committed)

## Step-by-Step

### 1. Prepare Release

```bash
# Ensure working directory is clean
git status

# Bump version in Cargo.toml (if needed)
# Update CHANGELOG.md
```

### 2. Build and Push Docker Image

```bash
# Tag and push
VERSION=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version')
docker build -t registry.fly.io/skill-workshop:$VERSION -t registry.fly.io/skill-workshop:latest .
docker push registry.fly.io/skill-workshop:$VERSION
docker push registry.fly.io/skill-workshop:latest
```

### 3. Run Database Migrations

```bash
# Run pending migrations against production database
DATABASE_URL=$PROD_DATABASE_URL cargo sqlx migrate run
```

### 4. Deploy to Fly.io

```bash
fly deploy --image registry.fly.io/skill-workshop:$VERSION
```

### 5. Verify Deployment

```bash
# Check health endpoint
curl -s https://skill-workshop.fly.dev/api/v1/health/ready

# Check readiness with full checks
curl -s https://skill-workshop.fly.dev/api/v1/health/ready | jq .

# Check metrics endpoint
curl -s -H "X-Metrics-Key: $METRICS_API_KEY" https://skill-workshop.fly.dev/metrics | head -20
```

### 6. Smoke Tests

```bash
# Run smoke tests
BASE_URL=https://skill-workshop.fly.dev benchmarks/smoke-test.sh
```

### 7. Monitor

- [ ] Grafana dashboard shows new version
- [ ] Error rate is normal (<1%)
- [ ] Latency is within thresholds
- [ ] No `5xx` errors in logs

## Rollback

If the deployment fails, follow [rollback.md](rollback.md).

## Post-Deployment

- [ ] Tag the release: `git tag v1.0.0 && git push --tags`
- [ ] Update CHANGELOG.md
- [ ] Notify the team in Slack
