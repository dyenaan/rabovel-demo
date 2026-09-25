# Current order-to-settlement UI flow

## Implemented behavior

1. The trading page loads market/portfolio data through feature hooks. `features/trading/components/order-form.tsx` reads eligibility from investor fixtures, validates with `schemas/order.schema.ts` and calculates a displayed total.
2. Submission creates an idempotency key and calls `usePlaceOrder` with market ID/symbol, side, order type, quantity and optional limit price. The key's presence does not prove server duplicate protection.
3. In mock mode, `features/orders/api/place-order.ts` inserts an OPEN order into the mutable `mockOrders` array with zero executed quantity. It creates no trade, portfolio update or settlement. These changes are not durable.
4. In HTTP mode, the same adapter posts to `/orders`, sends the idempotency header and expects `{ data: Order }`. This is a frontend contract expectation, not verified compatibility with the existing backend.
5. Success invalidates order queries, shows a toast and resets the form. Cancellation changes a mock record to CANCELLED or posts to `/orders/{id}/cancel`; it does not reverse a demonstrated on-chain transaction.
6. Trade and settlement screens independently query their fixtures or HTTP endpoints. `get-settlements.ts` reads `/settlements` and `/settlements/{id}`; there is no order-to-chain orchestration in the frontend.
7. Settlement detail displays lifecycle and derives finality from status. SUBMITTED/retry/replacement/blockhash-expiry display as SUBMITTED; INCLUDED and SAFE retain those labels; FINALIZED/RECONCILED/SETTLED display as FINALIZED; REORGED displays as ORPHANED; other statuses return no finality state. This mapping does not verify a chain transaction.

`src/types/settlement.types.ts` enumerates preparation, signature, submission, confirmation, hold and recovery states. `settlement-timeline.tsx` renders progression from status, not a fetched execution-event history. Settlement hooks do not currently subscribe to settlement WebSocket events, and the mock transport does not emit them despite declaring the channel name.

Primary-market subscriptions/redemptions are separate mutations under `features/primary-market`; mock SUBMITTED records do not progress through issuance/redemption automatically.

## Intended integration surfaces

The HTTP branches, WebSocket transport, status contracts and idempotency headers are scaffolding for backend integration. No durable product roadmap is present in this frontend; these interfaces should not be described as an agreed execution design. The sibling backend also has stub trading/settlement workers; see its [current lifecycle](../../rabovel-BE/docs/settlement-flow.md) only for cross-repository integration work.

## Unresolved

Actual API schemas/authentication, order-to-trade-to-settlement correlation, authorization, reservations and eligibility checks, duplicate protection, event ordering/reconnect recovery, status transitions and finality policy remain unverified. Do not translate a mock success toast or fixture SETTLED status into a claim of asset ownership or completed payment.
