CREATE TABLE password_credentials (
    user_id       TEXT PRIMARY KEY REFERENCES users(user_id) ON DELETE CASCADE,
    email         TEXT NOT NULL UNIQUE,
    display_name  TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
