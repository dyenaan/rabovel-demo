CREATE TABLE issuer_organizations (
    user_id                 TEXT PRIMARY KEY REFERENCES users(user_id) ON DELETE CASCADE,
    legal_name              TEXT NOT NULL,
    organization_type       TEXT NOT NULL,
    registration_number     TEXT NOT NULL,
    jurisdiction            TEXT NOT NULL,
    registered_address      TEXT NOT NULL,
    representative_name     TEXT NOT NULL,
    representative_title    TEXT NOT NULL,
    document_reference      TEXT NOT NULL,
    approved_at             BIGINT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
