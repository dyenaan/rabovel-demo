# Rabovel

Rabovel is a brokerage backend for tokenized assets, written in Rust. Runnable processes live under `apps/`, while reusable domain and adapter code lives under `crates/`. The HTTP API handles Google sign-in, wallet linking, KYC onboarding and trading authorization; three worker applications provide the event-consumer entry points.

This document is for engineers working in the repository. It describes what exists, how to run it, and, just as importantly, what does not exist yet.

---

## Status at a glance

| Capability | State |
|---|---|
| Google OIDC sign-in with PKCE and nonce | Working |
| Sign-In-With-Ethereum (SIWE) wallet linking | Working |
| Session tokens, rate limiting, OIDC and wallet challenges (Redis) | Working |
| Durable users, wallets, KYC cases (Postgres) | Working |
| KYC case submission and signed-webhook verdicts | Working against a mock provider |
| Kafka event publishing | Working, fire-and-forget (see [Known gaps](#known-gaps)) |
| Portfolio and order endpoints | Stub. Orders are acknowledged but not persisted or matched |
| Settlement, reconciliation, tokenization workers | Consumer plumbing only. They log events and do nothing else |
| Token-2022 mint setup, Token ACL preparation, shared-list setup | Reusable adapter and local commands; see [blockchain foundation](demo/README.md) |
| Matching engine, risk engine, smart order routing, worker-to-chain integration | Not started |

If you are here to build one of the workers, start at [Workers](#workers) and [Events](#events). If you are here to work on the API, start at [HTTP API](#http-api).

---

## Architecture

```mermaid
flowchart LR
    Client -->|HTTPS :3000| WAF[OWASP ModSecurity WAF]
    WAF --> GW[rabovel-api :8080]
    GW --> PG[(Postgres 16)]
    GW --> RD[(Redis 7)]
    GW -->|publish| K[(Kafka 3.8 KRaft)]
    GW -.->|OIDC| Google
    GW -.->|submit case, webhook| KYC[KYC provider, mock today]
    K --> S[settlement-worker]
    K --> R[reconciliation-worker]
    K --> T[tokenization-worker]
```

The API is a backend-for-frontend. It owns identity, onboarding and authorization. `events` defines transport-independent contracts; `infrastructure::messaging` supplies Kafka transport. See [architecture](docs/architecture.md).

### Crates

| Crate | Kind | Purpose |
|---|---|---|
| `domain` | library | The authentication and authorization model: `IdentityProfile`, `WalletConnection`, `BrokerageSecurityPolicy`, `BrokerageAuthService`. No I/O. |
| `kyc` | library | `KycProviderApi` trait plus `MockKycProvider`, which signs and verifies webhooks with HMAC-SHA256. |
| `portfolio-trading` | library | `PortfolioTradingApi` trait plus `StubPortfolioTradingService`. |
| `events` | library | Event envelopes, payload schemas and topic names. |
| `identity` | library | Identity repository records/contracts and provider ports. |
| `infrastructure` | library | Postgres, Redis, provider and Kafka adapters. |
| `rabovel-api` | application | Axum HTTP API and feature routes. |
| `settlement-worker`, `reconciliation-worker`, `tokenization-worker` | applications | Kafka consumer entry points. |
| Orchestrator crates | libraries | Worker business-service boundaries, currently skeletons. |
| `solana-adapter` | library + examples | Selected mint, authority and Token ACL foundation; not wired to workers yet. |

The API route groups live under `apps/rabovel-api/src/http/features`; process wiring stays in the application, while reusable implementations belong in crates.

---

## Blockchain foundation

Start with the [import scope and setup](demo/README.md) and [canonical on-chain reference](demo/ONCHAIN_REFERENCE.md). The adapter has no dependency on the former rabo-chain repository. Holder activation, issuance orchestration and settlement remain pending; the local commands are not a durable worker implementation.

## Quick start

### Prerequisites

- Rust 1.98 (the version pinned in the Dockerfile and CI)
- Docker and Docker Compose

### Configure

Create `.env` in the repository root. It is git-ignored. The full variable reference is in [Configuration](#configuration); the minimum is:

```dotenv
POSTGRES_USER=rabovel
POSTGRES_PASSWORD=change-me
POSTGRES_DB=rabovel
DATABASE_URL=postgres://rabovel:change-me@postgres:5432/rabovel

REDIS_PASSWORD=change-me
REDIS_URL=redis://:change-me@redis:6379/0

KAFKA_BOOTSTRAP_SERVERS=kafka:9092

RABOVEL_PUBLIC_ORIGIN=http://localhost:3001

# Required to publish approved asset metadata.
SUPABASE_URL=https://your-project-ref.supabase.co
SUPABASE_SECRET_KEY=sb_secret_replace-me

# Required together to enable issuer-triggered mint and ACL setup.
RABOVEL_SOLANA_RPC_URL=https://api.devnet.solana.com
RABOVEL_SOLANA_NETWORK=devnet
RABOVEL_ADMIN_KEYPAIR_BASE64=base64_of_admin_keypair_json
RABOVEL_BROKER_KEYPAIR_BASE64=base64_of_broker_keypair_json
RABOVEL_ISSUER_KEYPAIR_BASE64=base64_of_issuer_keypair_json
RABOVEL_MINT_DERIVATION_SECRET=64_or_more_hex_characters

# CNGN reuses the Solana RPC/network and derives broker owner from its keypair.
RABOVEL_CNGN_MINT_ADDRESS=HfJWS8vJHvxKn5xW3uLXkTmEy4jny3G45QnS1Eab5sg

# Optional trusted demo accounts. All other registrations become traders.
RABOVEL_DEMO_ISSUER_EMAIL=issuer@example.com
RABOVEL_DEMO_ADMIN_EMAIL=admin@example.com
```

### Run the whole stack

```bash
docker compose up -d --build
curl -s http://localhost:3000/health
```

Compose starts Postgres, Redis, Kafka, the API, and the WAF. The API has no public port of its own. Host port 3000 is the WAF, which proxies to internal port 8080. Migrations run automatically when the API starts.

### Run the API from source

The binary reads the process environment only. It does not load `.env` itself, so export the variables and point the service hostnames at localhost:

```bash
docker compose up -d postgres redis kafka

set -a; . ./.env; set +a
export DATABASE_URL=postgres://rabovel:change-me@localhost:5432/rabovel
export REDIS_URL=redis://:change-me@localhost:6379/0
export KAFKA_BOOTSTRAP_SERVERS=localhost:9092

cargo run -p rabovel-api
```

In a debug build, Swagger UI is served at `http://localhost:3000/swagger-ui` and the OpenAPI document at `/api-docs/openapi.json`. Both are compiled out of release builds.

### Run a worker from source

```bash
RUST_LOG=info KAFKA_BOOTSTRAP_SERVERS=localhost:9092 cargo run -p settlement-worker
```

---

## Configuration

All configuration is by environment variable. The API reads it during startup. `RABOVEL_BIND_ADDR` defaults to `0.0.0.0:8080`.

For hosted signing, Base64-encode the complete JSON files produced by `solana-keygen`; do not encode a seed phrase or public key. On GNU/Linux:

```bash
base64 -w 0 admin-keypair.json
base64 -w 0 broker-keypair.json
base64 -w 0 issuer-keypair.json
```

On macOS use `base64 < file.json`. Store the outputs only in the corresponding Railway secret variables. Base64 is transport encoding, not encryption; Railway access still grants access to these signing keys. The API derives stable ACL list seeds from the network and Admin public key, so hosted deployments need no ACL configuration variable.

### Authentication

| Variable | Notes |
|---|---|
| `RABOVEL_PUBLIC_ORIGIN` | Absolute URL of the public origin. Must be `https://` unless the host is `localhost`. Its authority becomes the SIWE domain. |

`RABOVEL_PUBLIC_ORIGIN` is also the single browser origin allowed by CORS. For a
local frontend on port 3001, use `http://localhost:3001`.

The frontend demo uses the persistent email/password endpoints. Legacy Google
OIDC routes remain fail-closed and are not initialized during startup.

### Demo role assignments

Public registration always defaults to `trader`; roles are never accepted from
an authentication request. To bind the two privileged demo accounts:

```dotenv
RABOVEL_DEMO_ISSUER_EMAIL=issuer@example.com
RABOVEL_DEMO_ADMIN_EMAIL=admin@example.com
```

A configured normalized email receives exactly the assigned `issuer` or `admin` role on sign-in;
issuer status does not imply KYB and trader status does not imply KYC.

### Optional backends

These select a durable backend when present. **When absent, the gateway silently falls back to an in-memory implementation.** It starts normally, accepts writes, and loses everything on restart. There is no warning. If persistence looks broken, check these first.

| Variable | When set | When unset |
|---|---|---|
| `DATABASE_URL` | Postgres via sqlx. Migrations run at startup. | `InMemoryRepository` |
| `REDIS_URL` | Sessions, challenges, and rate limits in Redis with native TTL. | In-memory store and limiter |
| `KAFKA_BOOTSTRAP_SERVERS` | Comma-separated broker list. Events are published to Kafka. | No-op producer, events are dropped |
| `SUPABASE_URL` + `SUPABASE_SECRET_KEY` | Approved asset metadata is generated and uploaded to the public `metadata` bucket. Both variables must be set together. | Draft approval fails closed with “metadata storage is not configured.” |
| `RABOVEL_SOLANA_RPC_URL` + `RABOVEL_SOLANA_NETWORK` + `RABOVEL_ADMIN_KEYPAIR_BASE64` + `RABOVEL_BROKER_KEYPAIR_BASE64` + `RABOVEL_ISSUER_KEYPAIR_BASE64` + `RABOVEL_MINT_DERIVATION_SECRET` | Enables replay-safe issuer mint/ACL setup using Base64-encoded Railway secrets. Keypair values must encode standard 64-byte Solana keypair JSON arrays. ACL configuration is derived from the network and Admin public key. All six variables must be set together. | Setup requests fail closed without creating an operation. |
| `RABOVEL_CNGN_MINT_ADDRESS` | Uses the primary Solana RPC/network for live SPL mint/program/decimal/supply verification. The broker owner is derived from its keypair. An approved issuer can idempotently create the broker ATA through `POST /payment-assets/cngn`; Rabovel Admin pays, and status reports the derived ATA and balance. | The CNGN rail reports not ready. |
| `RABOVEL_DEMO_MODE` | Set to `true` only for the demo deployment to bypass unfinished email/phone/KYC/terms/MFA/device onboarding checks. Trader role, action permission, linked-wallet ownership, balances, inventory and Phantom transaction signatures remain enforced. | Strict production onboarding policy remains active. |

### Optional tuning

| Variable | Default | Notes |
|---|---|---|
| `KYC_WEBHOOK_SHARED_SECRET` | random per process | HMAC key for the mock KYC webhook. Only the debug sandbox route uses it, so a random key is fine locally. |
| `WAF_PARANOIA_LEVEL` | `1` | OWASP CRS paranoia level for the compose WAF. |
| `POSTGRES_PORT`, `REDIS_PORT`, `KAFKA_PORT` | `5432`, `6379`, `9092` | Host ports published by compose. |
| `RUST_LOG` | unset | Log filter for the workers. The gateway does not emit logs yet. |

### Fixed limits

These limits remain application constants, not environment configuration.

| Limit | Value |
|---|---|
| Session lifetime | 900 seconds |
| OIDC challenge lifetime | 300 seconds |
| Wallet challenge lifetime | 300 seconds |
| Per-session rate limit | 10 requests per 60 seconds |
| Per-IP rate limit | 60 requests per 60 seconds |
| Request body limit | 16 KiB |

---

## HTTP API

All routes are served on port 3000. Authenticated routes expect `Authorization: Bearer <access_token>`, where the token comes from the Google callback. Request bodies reject unknown fields.

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/health` | no | Liveness. |
| GET | `/auth/google/start` | no | Begin Google sign-in. |
| POST | `/auth/google/callback` | no | Exchange the code for a session. |
| GET | `/onboarding/status` | yes | Current profile and onboarding status. |
| POST | `/onboarding/terms` | yes | Record terms acceptance. |
| POST | `/wallet/challenges` | yes | Issue a SIWE message to sign. |
| POST | `/wallet/link` | yes | Verify the signature and link the wallet. |
| POST | `/wallet/authorize` | yes | Ask whether a wallet action is permitted. |
| POST | `/kyc/cases` | yes | Submit a KYC case to the provider. |
| POST | `/kyc/webhook` | HMAC | Receive a provider verdict. |
| GET | `/portfolio` | yes, `view:portfolio` | Portfolio snapshot. Stub returns empty balances. |
| POST | `/orders` | yes, `trade:spot` | Place an order. Stub acknowledges only. |
| GET | `/orders` | yes, `view:portfolio` | List orders. Stub returns empty. |
| DELETE | `/orders/{order_id}` | yes, `trade:spot` | Cancel an order. Stub always succeeds. |
| POST | `/tokenization/requests` | yes, `trade:spot` | Request tokenization of an asset. |
| GET | `/tokenization/requests/{request_id}` | yes | Tokenization request status. |
| POST | `/kyc/_sandbox/advance` | no, **debug builds only** | Force a KYC verdict for a case. |

The third column names the wallet action each trading route is gated on. See [Authorization model](#authorization-model) for what that means in practice.

### Sign in

```bash
# 1. Get an authorization URL. Open it in a browser and sign in with Google.
curl -s http://localhost:3000/auth/google/start
# {"authorization_url":"https://accounts.google.com/...","state":"...","expires_in":300}

# 2. Google redirects to GOOGLE_REDIRECT_URI?code=...&state=...
#    Post both values back.
curl -s -X POST http://localhost:3000/auth/google/callback \
  -H 'content-type: application/json' \
  -d '{"code":"<code>","state":"<state>"}'
# {"access_token":"...","token_type":"Bearer","expires_in":900,
#  "profile":{...},"status":"requires_review"}
```

The state value is single-use and bound to a PKCE verifier and nonce stored server-side for 300 seconds. The email must be verified by Google or sign-in is refused. A new user is created on first sign-in and keyed by the OIDC issuer and subject, so a changed email does not create a second account.

### Link a wallet

```bash
TOKEN=...

# 1. Request a challenge. chain_id is EIP-155: 1 Ethereum, 8453 Base, 137 Polygon.
curl -s -X POST http://localhost:3000/wallet/challenges \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"address":"0xabc...","chain_id":1,"provider":"meta_mask"}'
# {"challenge_id":"...","message":"localhost:3000 wants you to sign in...","expires_at":...}

# 2. Sign `message` with the wallet (EIP-191 personal_sign), then link.
curl -s -X POST http://localhost:3000/wallet/link \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"challenge_id":"...","signature":"0x..."}'
# {"wallet":{...},"onboarding_status":"requires_review"}
```

Challenges are bound to the session that requested them and are single-use. The SIWE domain is derived from `RABOVEL_PUBLIC_ORIGIN`, so a message signed for one origin cannot be replayed against another. Providers are `meta_mask`, `wallet_connect`, `coinbase`, and `phantom`. Each chain-and-address pair can be linked to one user.

### Submit KYC

```bash
curl -s -X POST http://localhost:3000/kyc/cases \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"full_name":"Ada Lovelace","date_of_birth":"1815-12-10","country":"GB",
       "document_type":"passport","document_reference":"vendor-upload-ref-123"}'
# {"case_id":"...","submitted_at":...}
```

`document_reference` is a pointer into the vendor's own upload widget, never document bytes. The gateway deliberately does not handle file uploads.

The verdict arrives later on `POST /kyc/webhook` with an `x-kyc-signature: sha256=<hex>` header. The body is verified with HMAC-SHA256 against the shared secret before it is parsed, the case must exist and belong to the user named in the verdict, and only then is the profile updated. Document types are `passport`, `national_id`, and `drivers_license`.

In a debug build you can skip the provider round-trip:

```bash
curl -s -X POST http://localhost:3000/kyc/_sandbox/advance \
  -H 'content-type: application/json' -d '{"case_id":"..."}'
```

This builds a correctly signed approval webhook for the case and feeds it through the real webhook handler, so the same validation runs. The route is unauthenticated and exists only under `cfg(debug_assertions)`.

### Check status and accept terms

```bash
curl -s http://localhost:3000/onboarding/status -H "authorization: Bearer $TOKEN"

curl -s -X POST http://localhost:3000/onboarding/terms \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"version":"2026-09"}'
```

---

## Authorization model

Everything in this section lives in `crates/domain/src/auth.rs` and has no I/O, so it is the easiest part of the system to unit test.

### Trust boundary

The central rule is that **anything that grants trust must be constructed server-side from verified evidence, never deserialized from a request.** `VerifiedGoogleIdentity` can only be built from claims after token validation and refuses an unverified email. `VerifiedComplianceEvidence` can only be built from a webhook whose signature has already passed. Both have private fields. Request bodies can carry a `document_reference` or a `signature`, but never a `kyc_verified: true`.

If you add a new source of trust, follow the same shape: a private-field type with a constructor that takes the proof.

### Onboarding status

Every response that touches the profile includes one of three statuses, computed on the fly by `BrokerageAuthService::onboarding_status`:

- `blocked` if the risk tier is in the policy's blocking set. This is checked first and cannot be masked by anything else.
- `requires_review` if any required check is unmet.
- `approved` otherwise.

With the default `BrokerageSecurityPolicy`, `approved` requires all of the following:

| Requirement | How it becomes true |
|---|---|
| Email verified | Google sign-in with a verified email. |
| Risk tier assessed | KYC verdict. |
| Phone verified | KYC verdict. |
| KYC verified | KYC verdict. |
| Terms accepted | `POST /onboarding/terms`. |
| At least one wallet | `POST /wallet/link`. |
| MFA enrolled | **No endpoint exists yet.** |
| Trusted device | **No endpoint exists yet.** |

Because the last two cannot currently be satisfied, **no user can reach `approved` in the running system today**, and every financial action is refused with `onboarding_incomplete`. This is the single most important gap for anyone testing the trading path. See [Known gaps](#known-gaps).

### Wallet actions

Trading routes are gated on a `WalletAction`:

| Action | Financial | Needs fresh MFA |
|---|---|---|
| `view:portfolio` | no | no |
| `trade:spot` | yes | no |
| `trade:margin` | yes | yes |
| `sign:withdrawal` | yes | yes |
| `stake:yield` | yes | yes |

`authorize_wallet_action` checks, in order: the wallet belongs to the principal, the action is in the policy's allowed set, financial actions require the `trader` role, financial actions require `approved` onboarding, and fresh-MFA actions require an MFA timestamp within the policy window. Roles are `trader`, `compliance_officer`, `admin`, and `risk_ops`. Risk tiers are `unassessed`, `standard`, `enhanced`, and `high_risk`, with `high_risk` blocking by default.

---

## Data and state

### Postgres

Durable, queryable state. Four tables, defined in `migrations/0001_init.sql`:

- `users`: profile flags, risk tier, roles, permissions.
- `identity_links`: `(issuer, subject) -> user_id`, so one Google identity maps to exactly one user.
- `wallets`: linked wallets, unique per `(chain, address)`.
- `kyc_cases`: case status, risk tier, and the raw verdict JSON.

Migrations are embedded at compile time by `sqlx::migrate!` and applied at startup when `DATABASE_URL` is set. sqlx records what it has applied in `_sqlx_migrations`, so restarts are safe.

To add a migration, create the next numbered file in the `migrations` directory, then **rebuild**. Editing SQL without rebuilding changes nothing. There are no down migrations today.

Queries use runtime `query` and `query_as` rather than the compile-time-checked `query!` macros. This is deliberate: building the crate never needs a database or a `.sqlx` cache. The tradeoff is that a column typo is a runtime error, so integration tests against a real Postgres are the safety net.

Useful commands:

```bash
docker compose exec postgres psql -U "$POSTGRES_USER" -d "$POSTGRES_DB"
docker compose exec postgres psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c '\dt'
docker compose exec postgres psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" \
  -c 'SELECT version, description, success FROM _sqlx_migrations ORDER BY version;'
```

`docker compose down` keeps data in the `postgres_data` volume. `docker compose down -v` deletes it, along with the Redis and Kafka volumes.

### Redis

Everything with a TTL: sessions, pending OIDC state, used OIDC state, wallet challenges, and rate-limit token buckets. Redis expiry replaces manual cleanup. Nothing in Redis is a source of truth. Losing it logs everyone out and nothing else.

---

## Events

The `events` crate is the only thing downstream services share with the gateway.

### Envelope

Every message on every topic is JSON of this shape:

```json
{
  "event_id": "…",
  "event_type": "kyc.verdict-received",
  "occurred_at": 1758300000,
  "producer": "order-gateway",
  "schema_version": 1,
  "payload": { }
}
```

Payload types implement `DomainEvent`, which binds each one to an `EVENT_TYPE` and a `TOPIC`. Topic names carry a version suffix. A breaking payload change ships as a new `.v2` topic rather than mutating `.v1` under existing consumers.

### Topics

| Topic | Payload | Produced by |
|---|---|---|
| `rabovel.onboarding.status-changed.v1` | `OnboardingStatusChanged` | gateway, on any status transition |
| `rabovel.wallet.linked.v1` | `WalletLinked` | gateway |
| `rabovel.kyc.case-submitted.v1` | `KycCaseSubmitted` | gateway |
| `rabovel.kyc.verdict-received.v1` | `KycVerdictReceived` | gateway |
| `rabovel.trading.order-accepted.v1` | `OrderAccepted` | gateway |
| `rabovel.trading.trade-executed.v1` | `TradeExecuted` | nobody yet |
| `rabovel.portfolio.changed.v1` | `PortfolioChanged` | nobody yet |
| `rabovel.tokenization.requested.v1` | `TokenizationRequested` | gateway |
| `rabovel.tokenization.completed.v1` | `TokenizationCompleted` | nobody yet |
| `rabovel.tokenization.failed.v1` | `TokenizationFailed` | nobody yet |

`events::topics::ALL_TOPICS` lists them all for provisioning. The local Kafka container auto-creates topics; a production cluster should pre-create them with real partition and replication settings.

### Delivery guarantees, honestly

- **Publishing is fire-and-forget.** The gateway commits to Postgres, then calls `publish` and discards the result. If Kafka is unavailable the event is lost silently. A transactional outbox is the intended fix.
- **Consumers have no offset tracking.** `rskafka` has no consumer groups by design. `TopicConsumer` reads partition 0 from `Earliest` on every start. A worker that does real work must persist its last offset and pass `StartOffset::At(offset + 1)` on reconnect, or it will reprocess history after every restart.
- **Single partition.** Consumers cannot yet scale horizontally.

---

## Workers

The three worker applications are intentionally thin. Each initializes tracing, reads `KAFKA_BOOTSTRAP_SERVERS`, and spawns an `infrastructure::messaging` consumer task per topic. The corresponding orchestrator crates remain the business boundaries.

| Worker | Subscribes to |
|---|---|
| `settlement-worker` | `trade-executed`, `tokenization.completed` |
| `reconciliation-worker` | `trade-executed`, `portfolio.changed`, `tokenization.completed` |
| `tokenization-worker` | `tokenization.requested` |

To give a worker real behavior, replace the `log_events` call with your own loop over `TopicConsumer::next_event`. Keep the connect and decode plumbing; add offset persistence before anything with side effects.

The workers are not included in the Dockerfile or `docker-compose.yml` yet. Only the gateway is containerized.

---

## Development

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

CI runs formatting, Clippy and workspace tests on Rust 1.98, then builds the gateway Docker image. The workflow is in `.github/workflows/ci.yml`. Default tests require no running services; the adapter local-validator test is explicitly ignored unless selected.

### Conventions worth knowing

- **`deny_unknown_fields` on every request body.** A client that sends a field the server does not understand gets a 400, not silent acceptance.
- **Debug-only routes use `#[cfg(debug_assertions)]`, not a runtime flag.** The sandbox handler does not exist in a release binary, so it cannot be enabled by mistake.
- **No `unwrap`, `expect`, or `panic` in gateway request paths.** Keep it that way. Errors flow through `ApiError`, which maps domain, KYC, and trading errors to status codes.
- **Enum columns round-trip through serde.** `pg_repository` encodes enums to their `snake_case` serde names rather than hand-maintaining string tables.
- **Downstream crates never import `domain`.** If a worker needs a value, it belongs in an event payload.

### Docker

The image is a two-stage build. It copies `Cargo.lock` and builds with `--locked`, so an out-of-date lockfile fails the build rather than silently resolving different versions. It builds only `rabovel-api`. The `.dockerignore` excludes `target/`.

---

## Known gaps

Listed roughly in the order they should be addressed.

1. **No user can reach `approved`.** The default policy requires MFA enrollment and a trusted device, and no endpoint sets either. Until MFA and device-trust flows exist, either relax those two flags in a development policy or treat every trading route as unreachable.
2. **Event publishing is fire-and-forget.** Six publish sites discard errors. At minimum log them; properly, add an outbox table drained by a relay.
3. **Order IDs collide across restarts.** The stub trading service numbers orders from an in-process counter. Those IDs are already published in `OrderAccepted` events. Persisting orders with real IDs is the first step toward a trading path.
4. **The gateway emits no logs, traces, or metrics.** Two `println!` calls at startup are the entire observability story.
5. **Consumers replay history on restart.** Harmless while they only log. Add offset persistence before adding side effects.
6. **The trading, matching, risk, and worker-to-chain integration layers do not exist.** `portfolio-trading` is a stub and the other crates were removed until there is code to put in them.
7. **Workers are not deployed.** They are absent from compose and the Dockerfile.
8. **No down migrations.**
9. **Quantities should use a decimal type.** The stub parses quantity as `f64` only to validate positivity, and forwards the original string. When a real engine lands, use `rust_decimal`, never floating point, for quantities and prices.

---

## License

MIT
