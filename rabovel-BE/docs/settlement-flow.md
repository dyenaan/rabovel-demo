# Order-to-chain lifecycle

## Implemented now

1. The API authenticates the session and authorizes the order action (`place_order_handler` in `apps/rabovel-api/src/lib.rs`). Default onboarding currently prevents normal users from passing the financial gate; see [domain.md](domain.md).
2. If authorized, `StubPortfolioTradingService` validates quantity minimally and returns a process-local order ID. No order is persisted, matched or executed.
3. The gateway awaits publication of `OrderAccepted` through `events` and discards publication errors before returning the acknowledgment. A successful HTTP response does not establish durable event delivery.
4. The order path ends there. No current worker consumes `OrderAccepted` to execute trades, and no matching engine publishes `TradeExecuted`.
5. Separately, authorized tokenization requests publish `TokenizationRequested` and return an ID. The tokenization worker only logs requests; the HTTP status endpoint always says `pending`.
6. The settlement worker only logs `TradeExecuted` and `TokenizationCompleted`. Reconciliation only logs those plus `PortfolioChanged`. These subscriptions do not create settlement state or submit chain transactions.

The independent adapter can create and verify mints/shared lists and configure per-mint ACL state through RPC. No order/request is correlated to these operations; their transaction/readback flow is described in [solana.md](solana.md).

The investor catalog has a separate demo purchase path for explicitly live, backed listings. The API revalidates inventory and cNGN funds, prepares the investor equity account and ACL entry, and returns one partially issuer-signed Solana transaction containing both the investor-to-broker cNGN transfer and issuer-to-investor Token-2022 equity transfer. Phantom signs as the linked investor and fee payer; the API verifies all signatures and submits the transaction at confirmed commitment. This is a direct primary-inventory purchase, not matching-engine order settlement.

## Planned direction, not implemented

The [README gaps](../README.md#known-gaps) call for durable order IDs, an outbox and consumer progress tracking. The [on-chain reference, sections 9–10](../demo/ONCHAIN_REFERENCE.md#9-backend-mappings-and-durable-execution) describes durable operation preparation/submission/reconciliation and a possible future settlement-authorizing hook. Its proposed state names are guidance, not an existing backend state machine. Durable settlement tracking and reconciliation are not present in this workspace.

## Unresolved integration boundaries

- Durable settlement records, finality beyond confirmed commitment, failure recovery and cancellation.
- Durable operation/signature storage, economic duplicate protection and status propagation to the gateway.
- Consumer offsets and replay handling: `log_events` starts partition 0 at `Earliest`; `next_event` does not expose the consumed offset today. Adding persistence requires transport/interface work, not merely changing a worker start value.

Do not attach economic side effects to the logging loops without addressing replay and uncertain transaction outcomes. Solana prevents resubmitting an identical signed purchase transaction, but there is no durable idempotency key preventing an investor from intentionally preparing and signing another purchase.
