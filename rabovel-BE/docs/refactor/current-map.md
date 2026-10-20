# Current flow ownership

This is the implemented map for the whole-backend refactor authorized on 2026-10-20. The [baseline traces](flow-map.md) describe the starting behavior. [Intentional changes and retirements](retirements.md) qualify those traces; this map identifies the final owners. Durable trading, settlement recovery and reconciliation remain the separate D1–D5 upgrade track.

There are 3 runnable applications and 9 library crates. The HTTP surface has 59 explicit method/path pairs across 50 paths; 4 baseline methods were retired. Three of the 59 methods are development documentation/sandbox routes, compiled only with debug assertions and registered only without a gateway secret. Release therefore has 56 explicit methods across 47 paths. Implicit HEAD/OPTIONS handling is excluded.

The [HTTP register](current-routes.md) accounts for all H01–H63 individually. The [current inventory](current-inventory.json) assigns every Rust source under apps/crates to its supporting flows and lists test declarations. Refresh it with `python3 scripts/inspect_flows.py`. Generated metadata is a source snapshot, not a runtime discovery protocol or proof that every branch is tested. The six applied migrations and ten event topics remain; old tokenization topics have no active producer/consumer here.

## Every baseline flow

API paths below are relative to `apps/rabovel-api/src/features`. Library paths are relative to `crates`. HTTP modules contain routes/handlers/DTOs and transport-independent use cases; libraries contain shared models, rules, effect contracts and owned implementations where justified. Repositories/providers stay with the feature or cohesive group of workflows they serve.

| Flow | Final owner and disposition |
| --- | --- |
| F01 — Password accounts | `identity/accounts`; `identity/src/accounts/{passwords,ports,repositories}`. User + credential creation remains atomic; public roles remain server-assigned. |
| F02 — Session lifecycle | `identity/sessions`; identity session port/models and `identity/src/security_state`. HTTP bearer extraction stays in `http/authentication.rs`; TTL/hash/key contracts retained. |
| F03 — Google OIDC | Retired H02–H03 and live provider/state/configuration. Historical authentication discriminants, persisted identities and policy compatibility remain. |
| F04 — Financial authorization | `identity/access`, including actor and authoritative policy; identity access models/policy/rate limiter. Trading consumes the shared policy through `trading/orders/providers/access.rs`. |
| F05 — Terms/onboarding | `compliance/onboarding`; identity profile/status policy and profile-only updates. No role/permission writes from compliance. |
| F06 — Wallet linking/queries | `identity/wallets/{linking,queries}`; identity wallet models, challenge port, SQL/memory stores and SIWE provider. One-use challenge and ownership checks retained. |
| F07 — KYC | `compliance/kyc/{cases,verdicts,sandbox}`; `kyc/src/{cases,verdicts,ports,providers}`. Sandbox remains development-only; case/profile effects remain separate. |
| F08 — Issuer organizations | `issuer/organizations`; `issuer-assets/src/organizations/{models,ports,repositories}`. |
| F09 — Asset drafts | `issuer/assets/drafts`; `issuer-assets/src/assets/drafts/{models,ports,repositories}`. Ownership and guarded transitions retained. |
| F10 — Asset media/metadata | `issuer/assets/media`; `issuer-assets/src/assets/metadata/{ports,providers}`. Upload/confirm orchestration remains feature-owned. |
| F11 — Review/backing | `issuer/assets/{review,backing}` using draft repository transition operations and unchanged lifecycle rules. |
| F12 — Mint setup | `issuer/assets/issuance/setup`; `issuer-assets/src/assets/issuance/setup` models, chain port and operation stores. Issuer Solana provider owns setup execution. |
| F13 — Initial inventory | `issuer/assets/issuance/inventory`; issuer inventory models/port and issuer Solana inventory implementation. Setup and inventory remain distinct operations. |
| F14 — Listing publication | `issuer/assets/listings`; draft-owned guarded listing publication. |
| F15 — Broker cNGN readiness | `payments/cngn/broker_accounts`; cNGN models/port and shared payment providers. |
| F16 — Investor cNGN account | `payments/cngn/wallet_accounts`; same coherent payment provider capability, with authenticated-wallet ownership gates. |
| F17 — Instrument catalog | `catalog/instruments/{models,ports,resolvers,fixtures,providers}`. Simulated NGX and published-issuer resolution retain stable identifiers. |
| F18 — Markets/assets/charts/prices | `market_data/{markets,assets,charts,prices}`. Prices have local ports/providers; engine book/tape reads use market query interfaces. |
| F19 — Direct-purchase catalog | `settlement/direct_purchases/catalog`, consuming read-only asset/setup query contracts. |
| F20 — Quotes | `settlement/direct_purchases/quotes`; `settlement-orchestrator/src/direct_purchases/quotes` owns reusable checked pricing/configuration. Buy/sell quotation retains existing limitations. |
| F21 — Purchase preparation | `settlement/direct_purchases/preparation`; settlement preparation port/terms and the separate direct-purchase Solana provider. No durable intent/reservation added. |
| F22 — Purchase submission | `settlement/direct_purchases/submission`; settlement submission port and same purchase provider. Existing transaction checks and authority handling retained. |
| F23 — Order placement | `trading/orders`; `portfolio-trading/src/trading/orders/{ports,usecases}` and `trading/execution/engine_service`. Access/events are feature providers. |
| F24 — Order queries | Same order owner; `OrderQueries` is distinct from `OrderCommands`. One engine also supplies market query views. |
| F25 — Order cancellation | Same order owner; ownership/access policy and existing cancellation/event behavior retained. |
| F26 — Repricing | `trading/repricing/jobs.rs`; trading repricing port/use case and shared engine. Immediate first tick, five-second cadence and skipped missed ticks retained. |
| F27 — Portfolio | `portfolio/{balances,valuation,performance,activity}`; trading portfolio balance query plus issuer position port/Solana provider. Simulated balances and chain holdings remain distinguishable. |
| F28 — Legacy tokenization | Retired H48–H49, process and constant-only library. Event contracts retained for compatibility; no active completion workflow implied. |
| F29 — Health/development docs | `app/health.rs`, `http/documentation.rs`; same health and debug availability rules. |
| F30 — Composition/config/ingress | `app/{bootstrap,config,state,builder,persistence,chain,feature_ports,...}`, shared `http` ingress/authentication/errors/observation. Config validates once at startup; feature handlers receive narrow contexts. |
| F31 — Workers | `apps/{settlement-worker,reconciliation-worker}`. Both remain logging consumers. Tokenization subscriptions removed; no durable inbox/checkpoint or settlement execution claimed. |
| F32 — Event contracts/transport | `events` owns payloads/topics/envelopes, publishing port and serialization; `infrastructure/src/messaging` owns Kafka/no-op/memory implementations and consumers. Best-effort publication remains. |
| F33 — Solana primitives | `solana-adapter`, including `equity_setup_service`, `token_mint_builder` and shared account validation. Public Rust type names and chain instruction behavior retained. |
| F34 — Operator examples | Both `solana-adapter/examples` retained; demo authority configs point at external key files. Compilation does not establish validator/deployed compatibility. |
| F35 — Operations/schema | Postgres lifecycle/migrations in infrastructure; six migrations unchanged; host Cargo Watch + base Compose; effective CI at parent `.github/workflows/backend.yml`; historical docs archived, tracked keys externalized. |
| F36 — Matching rules | Pure `matching-engine` with `orders/{models,engine}`, book and house. Shared trading execution owns translation/locking; no network/storage dependency added. |
| F37 — Technical security state | Identity-owned independent session/challenge ports, shared memory/Redis security adapter, local rate limiter/proof provider. Business SQL and metadata removed from generic infrastructure. |

## Adding and changing flows

Start with the owning feature and lifecycle. Put a new operation beside related operations; add only responsibilities it actually needs. Handlers extract/map HTTP, use cases receive required capabilities through ports, and composition supplies a narrow context. API-specific orchestration can remain in the app without depending on Axum or `GatewayState`; lift it into a selective crate when another executable needs it. API capability getter traits bind contexts to application operations; public library ports describe the actual query/effect contract.

Use a read-only query interface for cross-feature reads. Introduce a named owner operation for writes; do not import another feature's SQL implementation. Keep transactions and guarded transitions explicit. Run the architecture guard and relevant behavior tests, then refresh the inventory when ownership/routes change. Folder names do not supply idempotency, recovery or transactionality.

## Verification boundaries

Normal workspace tests include authorization, route retirement, profile-only persistence, purchase/matching scenarios and quote arithmetic. Three infrastructure tests are opt-in: PostgreSQL migrations, PostgreSQL profile parity and local-validator minting. The deployment/client retirement window, Google-only accounts, actual chain/RPC behavior and secret rotation cannot be established by static source mapping. See the final verification record in the [campaign plan](../plans/feature-architecture-refactor.md).
