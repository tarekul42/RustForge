CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email CITEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    password_hash TEXT,
    phone TEXT,
    picture_url TEXT,
    age SMALLINT,
    address TEXT,
    role user_role NOT NULL DEFAULT 'student',
    status user_status NOT NULL DEFAULT 'active',
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    expertise TEXT,
    bio TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_role_status ON users (role, status);

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_token_hash ON sessions (token_hash);
CREATE INDEX idx_sessions_expires_at ON sessions (expires_at);

CREATE TABLE oauth_states (
    id UUID PRIMARY KEY,
    state TEXT UNIQUE NOT NULL,
    redirect_path TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oauth_states_state ON oauth_states (state);
CREATE INDEX idx_oauth_states_expires_at ON oauth_states (expires_at);

CREATE TABLE auth_codes (
    id UUID PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    payload JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_auth_codes_code ON auth_codes (code);
CREATE INDEX idx_auth_codes_expires_at ON auth_codes (expires_at);

CREATE TABLE otp_codes (
    id UUID PRIMARY KEY,
    email CITEXT NOT NULL,
    code_hash TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_otp_codes_email ON otp_codes (email);
CREATE INDEX idx_otp_codes_expires_at ON otp_codes (expires_at);
