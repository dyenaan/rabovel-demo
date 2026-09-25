# Rabovel backend

Rabovel is a Rust/Axum backend for a tokenized-equity demo. It supports the complete primary-market journey: account creation, wallet linking, simulated issuer approval and backing review, Token-2022 issuance, live listings, quotes, and atomic cNGN-for-equity purchases on Solana.

This is a working demo, not a production brokerage. KYC, backing verification and prices are simulated, and the tokens do not establish legal ownership of real shares.

## What is running today

```mermaid
flowchart LR
  FE[Next.js frontend] --> WAF[ModSecurity WAF :3000]
  WAF --> API[Axum API :8080]
  API --> PG[(Postgres)]
  API --> RD[(Redis)]
  API --> SOL[Solana RPC]
  API --> SB[Supabase metadata]
  API -. events .-> K[(Kafka)]
  K -.-> WK[Worker shells]
```

- **Identity and access:** email/password demo authentication, bearer sessions, trusted demo-role assignment, rate limiting, Solana wallet challenges and ownership proof. EVM SIWE is also supported; legacy Google OIDC routes are retained but are not initialized by the current startup composition.
- **Issuer lifecycle:** onboarding, asset drafts, simulated document-backed verification, metadata publishing, replay-safe mint setup, initial inventory issuance and explicit listing activation.
- **Investor lifecycle:** live-asset catalog, verified-backing summary, simulated price feed, wallet/cNGN/equity balance checks, expiring quotes and purchase settlement.
- **Persistence:** Postgres stores users, wallets, KYC cases, issuer assets and setup operations; Redis stores sessions, challenges and rate-limit state. In-memory fallbacks are used when these services are not configured.
- **Infrastructure:** Docker Compose runs Postgres, Redis, Kafka, the API and an OWASP ModSecurity reverse proxy. Debug builds expose Swagger UI.

## Solana integration

Each listed equity uses a zero-decimal Token-2022 mint. The API calls Solana directly through `solana-adapter`; Token ACL and ABL Gate are external programs, not Rabovel-owned on-chain programs.

1. **Mint setup:** create the mint account; initialize metadata-pointer, frozen-default, pausable and inactive transfer-hook extensions; call `InitializeMint2`; write token metadata.
2. **Eligibility setup:** create/verify shared allow and block lists, configure the mint ACL, delegate freeze authority to its MintConfig PDA and enable controlled permissionless thaw.
3. **Issuance:** allow-list the issuer wallet, create and thaw its associated token account, then `MintToChecked` the authorized whole-share supply into issuer inventory.
4. **Purchase preparation:** verify the linked investor wallet, cNGN funds and issuer inventory; allow-list the investor; create and thaw the investor equity ATA.
5. **Atomic settlement:** build one transaction containing `TransferChecked` from investor cNGN to the broker and `TransferChecked` from issuer inventory to the investor. The issuer partially signs the equity leg; Phantom signs as the linked investor and fee payer. The API verifies signatures and fee-payer ownership before submission and confirmation.

Token-2022 owns mint and token-account data. Token accounts record their wallet owner and mint; the API verifies expected program owner, token owner, mint and ATA derivation. ACL/Gate PDAs are owned by their respective programs. There is no permanent delegate with blanket power over holder balances.

See [Solana implementation notes](docs/solana.md) for the exact account and instruction model.

## Repository layout

| Path | Purpose |
| --- | --- |
| `apps/rabovel-api` | HTTP API, composition and Solana purchase orchestration |
| `apps/*-worker` | Kafka consumer entry points; business processing is not implemented |
| `crates/domain` | Identity, roles, onboarding and authorization rules |
| `crates/identity` / `crates/infrastructure` | Repository contracts and Postgres/Redis/Kafka/provider adapters |
| `crates/solana-adapter` | Token-2022 minting, authorities, ACL/Gate instructions and RPC verification |
| `crates/portfolio-trading` | Legacy portfolio/order contract with stub implementation |
| `migrations` | Postgres schema |

## Run locally

Prerequisites: Rust 1.98, Docker and Docker Compose.

Create a git-ignored `.env`:

```dotenv
POSTGRES_USER=rabovel
POSTGRES_PASSWORD=change-me
POSTGRES_DB=rabovel
DATABASE_URL=postgres://rabovel:change-me@postgres:5432/rabovel

REDIS_PASSWORD=change-me
REDIS_URL=redis://:change-me@redis:6379/0
KAFKA_BOOTSTRAP_SERVERS=kafka:9092
RABOVEL_PUBLIC_ORIGIN=http://localhost:3001

RABOVEL_DEMO_MODE=true
RABOVEL_DEMO_ISSUER_EMAIL=issuer@example.com
RABOVEL_DEMO_ADMIN_EMAIL=admin@example.com

SUPABASE_URL=https://your-project.supabase.co
SUPABASE_SECRET_KEY=replace-me

RABOVEL_SOLANA_RPC_URL=https://api.devnet.solana.com
RABOVEL_SOLANA_NETWORK=devnet
RABOVEL_ADMIN_KEYPAIR_BASE64=base64_keypair_json
RABOVEL_BROKER_KEYPAIR_BASE64=base64_keypair_json
RABOVEL_ISSUER_KEYPAIR_BASE64=base64_keypair_json
RABOVEL_MINT_DERIVATION_SECRET=at_least_64_hex_characters
RABOVEL_CNGN_MINT_ADDRESS=replace-with-mint
```

All six Solana setup variables must be supplied together. Base64 is encoding, not encryption; use development keys only and store them as deployment secrets.

```bash
docker compose up -d --build
curl -s http://localhost:3000/health
```

The WAF exposes port `3000` and proxies to the API on internal port `8080`. Migrations run at API startup. Debug builds provide `/swagger-ui` and `/api-docs/openapi.json`.

For source development and verification commands, see [docs/commands.md](docs/commands.md).

## Main API surfaces

| Area | Routes |
| --- | --- |
| Authentication | `/auth/register`, `/auth/login`, `/auth/me`, `/auth/logout` |
| Wallets/onboarding | `/wallet/*`, `/wallets`, `/onboarding/*`, `/kyc/*` |
| Issuers | `/issuer/overview`, `/issuer/onboarding`, `/issuer/assets/*` |
| Payment asset | `/payment-assets/cngn` |
| Investors | `/investor/catalog`, `/investor/quote`, `/investor/purchase/prepare`, `/investor/purchase/submit` |
| Legacy stubs | `/portfolio`, `/orders`, `/tokenization/requests` |

Authenticated routes use `Authorization: Bearer <token>`. The frontend demo should deploy with `RABOVEL_DEMO_MODE=true`: this bypasses onboarding checks that have no UI/endpoints yet, while preserving role, permission, linked-wallet, balance, inventory and transaction-signature checks.

## Known gaps

- Backing documents are metadata only; upload, custody evidence and admin review are simulated.
- KYC approval and the deterministic market-price feed are simulated.
- Purchase confirmation is real on-chain, but there is no durable purchase/settlement ledger, cost-basis history, reconciliation or automatic recovery after ambiguous RPC outcomes.
- `/portfolio` and `/orders` remain legacy stubs; the demo dashboard derives current holdings from live on-chain catalog balances.
- Kafka publishing is not transactional, and the settlement, reconciliation and tokenization workers are shells.
- Strict policy requires MFA and trusted-device state, but enrollment endpoints do not exist; demo mode intentionally relaxes those checks.
- Hosted keys are process secrets rather than HSM/MPC-backed production custody.
- The transfer-hook extension is inactive; current eligibility relies on frozen-by-default accounts, ACL allow/block lists and controlled thawing.
- The Solana path still requires a configured deployment, funded signers, program compatibility and end-to-end network smoke testing.

## Future state

A production system would integrate regulated KYC/KYB, document storage and custody attestations; licensed market data and pricing controls; HSM/MPC or qualified-custodian signing; a durable double-entry portfolio and settlement ledger; idempotent workers, outbox delivery and reconciliation; and an audited settlement program or active transfer hook with per-trade authorization and replay protection. It would also need secondary-market matching, corporate actions, surveillance, reporting, disaster recovery and jurisdiction-specific legal/regulatory approval.
