CREATE TABLE asset_drafts (
    asset_id             TEXT PRIMARY KEY,
    issuer_user_id       TEXT NOT NULL REFERENCES issuer_organizations(user_id) ON DELETE RESTRICT,
    canonical_key        TEXT NOT NULL,
    instrument_code      TEXT NOT NULL,
    draft                JSONB NOT NULL,
    status               TEXT NOT NULL CHECK (status IN ('draft', 'pending_review', 'approved_for_setup', 'rejected', 'revision_required')),
    created_at           BIGINT NOT NULL,
    updated_at           BIGINT NOT NULL,
    UNIQUE (issuer_user_id, canonical_key),
    UNIQUE (issuer_user_id, instrument_code)
);

CREATE INDEX asset_drafts_issuer_created_idx ON asset_drafts (issuer_user_id, created_at);
