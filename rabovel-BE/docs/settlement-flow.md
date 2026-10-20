# Execution and settlement boundaries

## Implemented paths

1. Order handlers call reusable order workflows after current identity/ownership/role/permission policy. Exact integer inputs and catalog prices feed the shared in-memory matching engine.
2. Acceptance precedes fill-level events. Publication is best-effort; errors preserve the acknowledgement. Repricing uses the same engine/event sink on an immediate/5-second schedule. Engine state, order IDs and balances do not recover across restart.
3. Settlement logs `TradeExecuted`. Reconciliation logs that plus `PortfolioChanged` (which has no active publisher). Neither worker moves assets, stores progress or repairs discrepancies.
4. Issuer mint/setup/inventory and direct investor purchases call chain providers independently. Purchase preparation may create/thaw accounts and add eligibility before returning a partially issuer-signed transaction. Investor signs as fee payer; submission checks that payer and signatures before confirmed RPC submission.

There is no engine-to-chain settlement integration. Quotes are not persisted/bound intents; sell quotes have no settlement endpoint. Purchase submission has no durable operation journal/status lookup/idempotency key. Existing setup readback/replay guards do not imply complete API crash recovery.

## Durability requirements

The [upgrade track](plans/feature-architecture-refactor.md#durable-upgrade-track) requires one mutation ordering authority, replayable commands/state, atomic execution facts/outbox, stable envelopes, consumer inbox/checkpoints, reservations, durable settlement attempts/finality/unknown outcomes, and reconciliation/projection rebuilds. Capture repricing mutations preceding a rejected new order as well as successful commands.

Kafka logging starts partition 0 at Earliest and discards record offsets from its returned interface. Do not attach economic effects without durable progress/deduplication and uncertain-outcome recovery. Repeating an identical signed transaction differs from creating/signing a new purchase; there is no durable business idempotency mechanism today.

The legacy tokenization API/worker were removed. Retained tokenization payloads/topics support compatibility, not an active processor. See [retirements](refactor/retirements.md), [Solana details](solana.md), [baseline traces](refactor/flow-map.md) and [current owners](refactor/current-map.md).
