CREATE TABLE reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    workshop_id UUID NOT NULL REFERENCES workshops(id) ON DELETE RESTRICT,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    title TEXT NOT NULL CHECK (length(title) <= 120),
    content TEXT NOT NULL CHECK (length(content) <= 2000),
    status review_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reviews_user_id ON reviews (user_id);
CREATE INDEX idx_reviews_workshop_id ON reviews (workshop_id);
CREATE INDEX idx_reviews_status ON reviews (status);
CREATE UNIQUE INDEX idx_reviews_user_workshop ON reviews (user_id, workshop_id);
CREATE INDEX idx_reviews_workshop_status_created ON reviews (workshop_id, status, created_at DESC);

CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL CHECK (length(name) <= 100),
    email CITEXT NOT NULL,
    subject TEXT NOT NULL CHECK (length(subject) <= 200),
    message TEXT NOT NULL CHECK (length(message) <= 5000),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contacts_unread_created ON contacts (is_read, created_at DESC);
CREATE INDEX idx_contacts_created ON contacts (created_at DESC);
