CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT UNIQUE NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    thumbnail_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE levels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE workshops (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    location TEXT,
    price_cents BIGINT NOT NULL CHECK (price_cents >= 0),
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    max_seats INTEGER CHECK (max_seats > 0),
    current_enrollments INTEGER NOT NULL DEFAULT 0 CHECK (current_enrollments >= 0),
    min_age SMALLINT,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    level_id UUID NOT NULL REFERENCES levels(id) ON DELETE RESTRICT,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workshops_slug ON workshops (slug);
CREATE INDEX idx_workshops_category_level_start ON workshops (category_id, level_id, start_date DESC);
CREATE INDEX idx_workshops_created_by ON workshops (created_by);

CREATE TABLE workshop_images (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workshop_id UUID NOT NULL REFERENCES workshops(id) ON DELETE RESTRICT,
    url TEXT NOT NULL,
    s3_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workshop_images_workshop_id ON workshop_images (workshop_id);
