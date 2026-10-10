# Backend architecture

Rabovel uses a Cargo workspace with two package groups:

- `apps/*` are runnable composition roots. They parse process configuration, assemble adapters and expose HTTP or consume events.
- `crates/*` are reusable business boundaries, contracts and infrastructure adapters.

## Components

| Component | Responsibility |
| --- | --- |
| `apps/rabovel-api` | Axum HTTP API. `http/features` owns route groups and `config` owns server configuration. |
| `apps/*-worker` | Process entry points for tokenization, settlement and reconciliation. |
| `domain` | I/O-free identity, onboarding and authorization rules. |
| `identity` | Identity repository contract, records and provider ports. |
| `kyc` | KYC provider contract and signed mock implementation. |
| `portfolio-trading` | Portfolio/order contract, the engine-backed `EngineTradingService` used by the API, and a test stub. |
| `matching-engine` | I/O-free principal-backed matching: client book plus house quote, risk limits, float/mint inventory and hedge netting. See [plans/principal-matching-engine.md](plans/principal-matching-engine.md). |
| Orchestrator crates | Business-service boundaries used by workers; currently skeletons. |
| `events` | Transport-independent envelopes, payloads and topic names. |
| `infrastructure` | Postgres, Redis, OIDC/SIWE, rate limiting, Kafka and in-memory adapters. |
| `solana-adapter` | Token-2022, Token ACL and ABL Gate instruction/RPC implementation. |

Dependency direction is `apps → domain crates + infrastructure`. Infrastructure implements ports owned by the relevant domain crate. Domain crates do not depend on applications or concrete infrastructure.

## Current execution

Local Compose runs only Postgres and Redis, with ports bound to `127.0.0.1`. The API runs on the host through Cargo Watch at `http://localhost:8080`; `RABOVEL_BIND_ADDR` overrides its bind address. Postgres holds durable identity/KYC data, and Redis holds sessions, one-use challenges and rate-limit state with a 25 MB memory limit and `noeviction`. The optional test container runs only when explicitly requested.

Kafka and the WAF are not part of the local stack. Leave `KAFKA_BOOTSTRAP_SERVERS` unset to select the no-op event producer; a separately configured Kafka broker enables the existing event adapter.

The Next.js gateway holds bearer sessions in HttpOnly cookies and forwards them server-to-server. The API requires a shared gateway secret for every route except `/health` when configured; hosted origins require it. Client IP is trusted only after gateway authentication. Without a secret, localhost development uses the socket peer and ignores forwarded headers. Debug documentation routes are disabled when the secret is configured. Workers still log events and start from the earliest offset; durable checkpoints and idempotent handling are required before they perform side effects. Event publication still lacks a transactional outbox.

The API calls the Solana adapter directly for mint setup, issuance and purchases; worker shells do not execute these operations. Validator-backed tests remain opt-in.
