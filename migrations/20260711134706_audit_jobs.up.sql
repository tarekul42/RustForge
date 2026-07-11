CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    changes JSONB NOT NULL DEFAULT '{}',
    ip_address INET,
    user_agent TEXT
);

CREATE INDEX idx_audit_logs_event_type ON audit_logs (event_type);
CREATE INDEX idx_audit_logs_aggregate ON audit_logs (aggregate_type, aggregate_id);
CREATE INDEX idx_audit_logs_aggregate_occurred ON audit_logs (aggregate_type, occurred_at DESC);
CREATE INDEX idx_audit_logs_actor_occurred ON audit_logs (actor_id, occurred_at DESC);
CREATE INDEX idx_audit_logs_occurred_at ON audit_logs (occurred_at DESC);

CREATE TABLE jobs (
    id BIGSERIAL PRIMARY KEY,
    job_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    run_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    locked_by UUID,
    locked_at TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_jobs_pending_run_at ON jobs (run_at) WHERE status = 'pending';
CREATE INDEX idx_jobs_status ON jobs (status);
CREATE INDEX idx_jobs_type_status ON jobs (job_type, status);

CREATE TRIGGER trigger_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trigger_categories_updated_at BEFORE UPDATE ON categories
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trigger_levels_updated_at BEFORE UPDATE ON levels
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trigger_workshops_updated_at BEFORE UPDATE ON workshops
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trigger_enrollments_updated_at BEFORE UPDATE ON enrollments
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trigger_payments_updated_at BEFORE UPDATE ON payments
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trigger_reviews_updated_at BEFORE UPDATE ON reviews
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER trigger_contacts_updated_at BEFORE UPDATE ON contacts
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
