ALTER TABLE asset_drafts DROP CONSTRAINT asset_drafts_status_check;
ALTER TABLE asset_drafts ADD CONSTRAINT asset_drafts_status_check CHECK (
    status IN ('draft', 'pending_review', 'approved_for_setup', 'minted', 'rejected', 'revision_required')
);
ALTER TABLE asset_drafts ADD COLUMN mint_address TEXT UNIQUE;

CREATE TABLE asset_setup_operations (
    operation_id            TEXT PRIMARY KEY,
    asset_id                TEXT NOT NULL UNIQUE REFERENCES asset_drafts(asset_id) ON DELETE RESTRICT,
    issuer_user_id          TEXT NOT NULL REFERENCES issuer_organizations(user_id) ON DELETE RESTRICT,
    network                 TEXT NOT NULL CHECK (network IN ('devnet', 'testnet')),
    mint_address            TEXT NOT NULL UNIQUE,
    status                  TEXT NOT NULL CHECK (status IN ('running', 'confirmed', 'reconciliation_required', 'failed')),
    stages                  JSONB NOT NULL,
    mint_config_address     TEXT,
    allow_list_address      TEXT,
    block_list_address      TEXT,
    thaw_extra_metas_address TEXT,
    error                   TEXT,
    created_at              BIGINT NOT NULL,
    updated_at              BIGINT NOT NULL
);
