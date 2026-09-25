-- Durable, queryable state for the Rabovel BFF. Ephemeral/TTL-bound state
-- (sessions, OIDC pending/used-state, wallet challenges) deliberately does
-- NOT live here -- it lives in Redis, where native EXPIRE/SETEX replaces
-- the manual .retain() cleanup this table set previously needed.

CREATE TABLE users (
    user_id                TEXT PRIMARY KEY,
    email                  TEXT NOT NULL,
    email_verified         BOOLEAN NOT NULL,
    phone_verified         BOOLEAN NOT NULL,
    kyc_verified           BOOLEAN NOT NULL,
    terms_version          TEXT NULL,
    mfa_enrolled           BOOLEAN NOT NULL,
    trusted_device         BOOLEAN NOT NULL,
    risk_tier              TEXT NOT NULL,
    compliance_checked_at  BIGINT NULL,
    auth_provider          TEXT NOT NULL,
    roles                  TEXT[] NOT NULL DEFAULT '{}',
    permissions            TEXT[] NOT NULL DEFAULT '{}',
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE identity_links (
    issuer   TEXT NOT NULL,
    subject  TEXT NOT NULL,
    user_id  TEXT NOT NULL REFERENCES users(user_id),
    PRIMARY KEY (issuer, subject)
);

CREATE TABLE wallets (
    wallet_id      TEXT PRIMARY KEY,
    owner_user_id  TEXT NOT NULL REFERENCES users(user_id),
    chain          TEXT NOT NULL,
    address        TEXT NOT NULL,
    provider       TEXT NOT NULL,
    verified_at    BIGINT NOT NULL,
    UNIQUE (chain, address)
);

CREATE INDEX wallets_owner_user_id_idx ON wallets (owner_user_id);

CREATE TABLE kyc_cases (
    case_id           TEXT PRIMARY KEY,
    user_id           TEXT NOT NULL REFERENCES users(user_id),
    status            TEXT NOT NULL,
    risk_tier         TEXT NULL,
    submitted_at      BIGINT NOT NULL,
    finalized_at      BIGINT NULL,
    raw_verdict_json  JSONB NULL
);

CREATE INDEX kyc_cases_user_id_idx ON kyc_cases (user_id);
