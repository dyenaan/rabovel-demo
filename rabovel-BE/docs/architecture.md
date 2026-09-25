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
| `portfolio-trading` | Portfolio/order contract and deterministic stub. |
| Orchestrator crates | Business-service boundaries used by workers; currently skeletons. |
| `events` | Transport-independent envelopes, payloads and topic names. |
| `infrastructure` | Postgres, Redis, OIDC/SIWE, rate limiting, Kafka and in-memory adapters. |
| `solana-adapter` | Token-2022, Token ACL and ABL Gate instruction/RPC implementation. |

Dependency direction is `apps → domain crates + infrastructure`. Infrastructure implements ports owned by the relevant domain crate. Domain crates do not depend on applications or concrete infrastructure.

## Current execution

The WAF exposes host port 3000 and proxies to `rabovel-api` on internal port 8080. `RABOVEL_BIND_ADDR` overrides the bind address. Postgres holds durable identity/KYC data, Redis holds sessions and one-use challenges, and Kafka carries integration events.

The API preserves the existing routes and bearer-session behavior. Workers still log events and start from the earliest offset; durable checkpoints and idempotent handling are required before they perform side effects. Event publication still lacks a transactional outbox.

The Solana adapter is not wired to the API or workers. Validator-backed tests remain opt-in.
