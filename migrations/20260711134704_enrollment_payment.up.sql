CREATE TABLE enrollments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    workshop_id UUID NOT NULL REFERENCES workshops(id) ON DELETE RESTRICT,
    payment_id UUID,
    student_count INTEGER NOT NULL CHECK (student_count > 0),
    status enrollment_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_enrollments_user_id ON enrollments (user_id);
CREATE INDEX idx_enrollments_workshop_id ON enrollments (workshop_id);
CREATE INDEX idx_enrollments_status ON enrollments (status);
CREATE UNIQUE INDEX idx_enrollments_user_workshop_active ON enrollments (user_id, workshop_id) WHERE status IN ('pending', 'complete');
CREATE INDEX idx_enrollments_user_created ON enrollments (user_id, created_at DESC);
CREATE INDEX idx_enrollments_status_created ON enrollments (status, created_at DESC);

CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    enrollment_id UUID UNIQUE NOT NULL REFERENCES enrollments(id) ON DELETE RESTRICT,
    transaction_id TEXT UNIQUE NOT NULL,
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),
    payment_gateway_data JSONB,
    invoice_url TEXT,
    status payment_status NOT NULL DEFAULT 'unpaid',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_enrollment_id ON payments (enrollment_id);
CREATE INDEX idx_payments_transaction_id ON payments (transaction_id);
CREATE INDEX idx_payments_status ON payments (status);

CREATE TABLE refund_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id UUID NOT NULL REFERENCES payments(id) ON DELETE RESTRICT,
    amount_cents BIGINT NOT NULL CHECK (amount_cents > 0),
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refund_logs_payment_id ON refund_logs (payment_id);
