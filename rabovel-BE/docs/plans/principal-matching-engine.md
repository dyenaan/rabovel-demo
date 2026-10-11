# Principal-backed trading: matching, settlement, hedging, liquidity

## Business model (target)

Rabovel acts as a principal dealer for tokenized equities. Clients get **instant fills and instant on-chain delivery**; Rabovel absorbs the timing risk and squares its exposure with the underlying market later.

- **Matching is off-chain.** Each order executes against the better of resting client orders (internalized) and the house quote (reference price ± spread). The house is the guaranteed counterparty within per-symbol risk limits.
- **Client settlement is on-chain and immediate.** Every fill moves equity tokens and cNGN on Solana right after matching. House sales deliver from the **pre-funded treasury float first**. Only the shortfall is **minted against the pending hedge**, capped by `max_minted_unbacked`. House purchases retire unbacked mints before refilling the float. (Decided 2026-10-05.)
- **Hedging and clearing are batched.** Principal fills are netted per symbol into a hedge window. At end of day or end of week, one instruction per symbol goes to the clearing house/broker and settles at T+N, outside the client's path.
- **Liquidity must always cover withdrawals.** Treasury cNGN and token float, minus pending hedge obligations, must stay above a reserve floor. The house quote widens or stops before that floor is breached.

## Slice 1 — matching engine core (done)

`crates/matching-engine` is pure, synchronous and deterministic, with no I/O or clock.

- `MatchingEngine::submit(order, reference_price)` executes book and house liquidity level by level, best price first. On a tie the book wins, which keeps the house flat. Market remainders are cancelled with an `UnfilledReason`, and limit remainders rest. Self-trade prevention cancels the submitter's own resting order.
- `reprice(symbol, reference)` lets the house fill resting limits that the new quote now satisfies, at the quote price.
- `HouseLimits` (per symbol, with a default) sets the half-spread, `max_long`, `max_short` and `max_order_quantity`. Capacity is derived from the house's net position.
- `close_hedge_window()` nets principal fills into a `HedgeInstruction` per symbol and resets the window. Positions carry over. Each instruction also reports `minted_in_window` and the outstanding `minted_unbacked` the clearing buy must cover.
- `Fill` has explicit `buyer`/`seller` (`Client` or `House`), integer quantity (whole tokens), integer price (cNGN base units), a source and a gap-free `sequence`. Principal fills also carry `house_inventory`: `HouseDelivers { from_float, minted }` or `HouseReceives { to_float, retired }`. The settlement worker maps these to treasury transfer, mint and burn instructions.
- `fund_float(symbol, qty)` pre-funds the treasury. `settle_hedge(symbol, side, qty)` applies a T+N clearing result: buys back unbacked mints first, then refill the float.
- Setting `max_minted_unbacked = 0` turns off minting, so the house delivers strictly from the float.

Verify with `docker compose run --rm tests` (20 tests, clippy-clean).

## Slice 2 — engine wired into the API (done)

- `EngineTradingService` (`crates/portfolio-trading/src/engine_service.rs`) implements `PortfolioTradingApi` over a mutex-guarded engine:
  - Quantity and price strings convert exactly to whole units and base units in `decimal.rs`, never through floats.
  - Each order first runs `reprice` with the current reference price, then `submit`.
  - It keeps per-user order records and net matched-trade balances. `locked` covers resting orders.
- `ReferencePriceSource` is the price port. `CatalogReferencePrices` (`apps/rabovel-api/src/price_feed.rs`) accepts only instrument-catalog tickers and prices them from `PriceFeed`. A price older than `max_price_age_secs` stops the house quoting; client-to-client matching continues.
- `POST /orders` returns an execution ack: state, filled/resting/cancelled quantities, average price, fills and unfilled reason. Counterparties are never included. `GET /orders/:order_id` was added. `GatewayStateBuilder` builds the engine service by default, and `RABOVEL_HOUSE_*` variables configure it (see README).
- `TradeExecuted` is now fill-level: buyer/seller (`client` or `house`), liquidity, sequence and `house_inventory`. The API publishes one per trade after `OrderAccepted`.
- Frontend: `rabovel-FE/src/features/orders/api/backend-orders.ts` maps the backend contract (bare tickers, snake_case, `DELETE /orders/:id`) onto the UI `Order` type. Mock mode is unchanged.

Verify with `docker compose run --rm tests cargo test -p matching-engine -p portfolio-trading -p rabovel-api -p events --locked`. The HTTP test is `orders_execute_against_the_matching_engine_and_publish_trades`.

Carried forward:
- **Pre-trade checks.** Buying power for buys and token balance for sells, reserved on submit and released on cancel. These need on-chain cNGN and token balances, so they come with slice 3.
- **Durability.** Engine state is in memory. Persist an order/fill journal (Postgres) and rebuild by replay using `sequence`. Event publication still discards errors; this needs the outbox.
- **Idempotency.** The frontend sends `Idempotency-Key`, but the backend ignores it.
- **Tradable symbols** are catalog instruments only. Minted live assets (e.g. `RABO-*`) are not tradable yet. This is superseded by [unified-tradable-assets.md](unified-tradable-assets.md), which also redefines slice 3 below to use delegated investor authority.

## Slice 3 — instant on-chain settlement (settlement-worker)

Consume `TradeExecuted`. For each fill, build one transaction that delivers tokens seller→buyer and cNGN buyer→seller. The house side uses the broker treasury accounts, reusing the purchase path in `mint_setup.submit_purchase`. Record each settlement durably (fill id → signature → status) for idempotency before submitting. Address the replay/offset gaps in [settlement-flow.md](../settlement-flow.md#unresolved-integration-boundaries) first.

## Slice 4 — hedge batch and clearing (T+N)

A scheduled job calls `close_hedge_window()`, persists the instructions, and sends them to the clearing broker (simulated). It tracks each instruction as `pending → executed → settled (T+N)`. Settled buys replenish backing for tokens already delivered.

## Slice 5 — reconciliation (reconciliation-worker)

Run three-way checks per symbol:

1. On-chain token supply and treasury balances against the ledger.
2. Engine house position against the sum of principal fills.
3. Delivered-but-unhedged units against open hedge instructions.

Alert on breaks, and halt house quoting for a symbol whose break exceeds tolerance.

## Slice 6 — liquidity guard and withdrawals

Track available liquidity: treasury cNGN + token float − pending hedge obligations − reserved withdrawals. Feed it into `HouseLimits`, either dynamically or by halting quotes below a floor. Withdrawals reserve against this pool and are approved only while it stays above the floor.

## Unresolved decisions (business/compliance)

- **Size of the float and the unbacked-mint cap** per instrument. Minted units are unbacked between trade and T+N, so `max_minted_unbacked` is a compliance limit as well as a risk limit.
- **Mint authority for settlement.** Mints currently have issuer mint authority (see [solana.md](../solana.md)), so slice 3 needs the house signer to be allowed to mint, or a delegated flow.
- **Reference price source.** Exchange/broker feed versus the current simulated feed, and the staleness tolerance.
- **Spread and limits** per instrument, and who can change them at runtime.
- **Partial-fill policy for market orders** when house capacity runs out: partial fill (current behavior) or all-or-none.
- **Regulatory treatment** of principal dealing and deferred clearing versus CFD-style contracts. This determines whether clients hold the token or a claim against Rabovel.
