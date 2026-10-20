# Refactor contract and cleanup record

Authorized 2026-10-20. These are intentional changes alongside structural moves, not new settlement guarantees.

## Google OIDC

`GET /auth/google/start` and `POST /auth/google/callback` have no registered handlers and return 404. Env composition previously left Google disabled. First-party auth uses passwords, and the frontend gateway already blocked Google start. Dedicated provider, nonce/PKCE state, configuration, trusted-role wiring and unused identity-creation persistence were removed. Isolated password-account fixtures retain the role/ownership scenarios.

Historical authentication discriminants, user records, identity-link tables and applied migrations remain compatible. Legacy verified-identity policy tests remain. Repository inspection does not establish external caller or Google-only account usage; deployment ownership must account for that limitation. Stored account identities were not rewritten.

## Legacy tokenization

`POST /tokenization/requests` and `GET /tokenization/requests/:request_id` are removed from backend and frontend gateway. They previously published a request and always reported pending, with no completion implementation. The logging tokenization worker/library were removed. Active issuer setup/inventory/backing/listing and direct purchases remain.

Legacy event schemas/topics remain compatibility contracts. Settlement/reconciliation no longer subscribe to tokenization completion; retained payload definitions do not imply an active processor.

## Startup validation

Quote fee/expiry settings now parse once at startup. Defaults remain 100 basis points and 300 seconds; accepted values/arithmetic are preserved. Invalid settings prevent startup. Bind configuration also validates before dependency initialization. Quote IDs/expiry timestamps still are not persisted purchase intents or enforced reservations.

## Profile persistence

Compliance and wallet linking now save profile fields through an identity-owned operation that does not update roles/permissions. Memory regression coverage verifies principal/credential preservation; optional PostgreSQL parity uses connection-local temporary tables. Credential+user creation remains atomic. Case/profile and chain/database effects remain separate operations.

## Repository and operations

- Removed `domain` after moving auth/policy/wallet models to identity and asset models to issuer.
- Reused settlement for purchase ports/pricing; removed reconciliation's constant-only wrapper while retaining its logging app.
- Retired incomplete `docker-compose.dev.yml`; host Cargo Watch plus base Compose is the supported local workflow.
- Moved CI to the actual Git root with correct backend context and gateway verification.
- Archived superseded design/import records in `docs/history`; current runbooks and fixtures remain.
- Normalized Solana paths to `equity_setup_service/token_mint_builder.rs`, preserving public type names.
- Preserved three local signing files under `/home/dyenaan/.config/rabovel/keys` with restricted permissions before removing repository copies. Existing destination files were preserved using `*-from-repo-20261020.json` names where needed. History and deployed secrets were not rewritten/rotated; rotation remains an operational task.

Docker, database and validator checks require their documented environments. Source/unit checks cannot establish network confirmation, recovery or deployed-client compatibility.
