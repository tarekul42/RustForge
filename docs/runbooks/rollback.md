# Rollback Runbook

## When to Roll Back

Roll back immediately if any of the following is true:

- **Error rate** > 5% for any critical endpoint
- **p99 latency** > 1s for read endpoints
- **Health check** returns non-200 for >30 seconds
- **Database migration** caused data loss or corruption
- **Critical feature** (auth, enrollment, payment) is broken

## Rollback Steps

### 1. Roll Back Database Migration (if applicable)

```bash
# If the last migration caused issues, run the down migration
# WARNING: Only run this if the migration is reversible and the
# data loss is acceptable. Most migrations are forward-only.
# Preferred approach: write a new migration to fix the issue.

# For forward-only migrations, skip this step and write a fix migration.
```

### 2. Revert to Previous Docker Image

```bash
# Deploy the previous version
fly deploy --image registry.fly.io/skill-workshop:PREVIOUS_VERSION
```

### 3. Verify Health

```bash
curl -s https://skill-workshop.fly.dev/api/v1/health/ready | jq .
```

### 4. Restore Database (if data was corrupted)

```bash
# Restore from the latest backup
# Assumes pg_dump backups are taken every 6 hours
pg_restore -d $PROD_DATABASE_URL -c latest_backup.dump
```

### 5. Notify

- Notify the team in Slack (#incidents channel)
- Open a GitHub issue describing the incident
- Update the runbook if the rollback revealed gaps

## After Rollback

- [ ] Determine root cause
- [ ] Write a regression test
- [ ] Deploy the fix with a new version (not a re-deploy of the rolled-back version)
- [ ] Schedule a post-mortem

## Prevention

| Cause | Prevention |
|-------|-----------|
| Database migration breaks | Test migrations against a staging copy of production data |
| Configuration error | Validate config in CI with `cargo run -- check-config` |
| Dependency vulnerability | `cargo audit` in CI |
| Performance regression | Load test in CI with baseline comparison |
