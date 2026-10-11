# Trading simulator: deposit, buy and sell like a trading app

**Plan — 2026-10-10.** Builds on [unified-tradable-assets.md](unified-tradable-assets.md) and absorbs its U2–U4. [principal-matching-engine.md](principal-matching-engine.md) still covers hedging and reconciliation.

## Outcome

An investor signs up, deposits demo cNGN into their own Solana wallet from inside the app, and buys and sells any catalog asset. The catalog is 146 NGX-listed instruments plus issuer listings, and it behaves like a trading app:

- live-moving prices and charts;
- market and limit orders;
- an order book and a trade tape;
- holdings and P&L that survive restarts.

## Decisions (2026-10-10)

1. **Money lives on-chain only.** Cash is the investor's demo cNGN on Solana Devnet. There is no off-chain cash ledger. Trades settle on-chain, using the delegation design in U2/U3 of the unified plan.
2. **Deposit is a Rabovel demo cNGN faucet.**
   - Rabovel creates its own demo cNGN mint, labelled a demo representation and never official cNGN.
   - The Admin key holds mint authority.
   - **Deposit** mints into the investor's cNGN account, capped per day.
   - "Withdraw" isn't needed, because funds never leave the investor's wallet.
3. **Persistence is in Postgres.** Orders, fills and settlement records are stored, and the engine rebuilds its book on startup.
4. **Prices are a simulated live market.** Each ticker follows a deterministic, smooth random-looking path around its base price, so history can be recomputed exactly after a restart. A background tick reprices the engine so resting limit orders fill as the market moves.

## Slices

| # | Slice | Needs Solana config | Result |
| --- | --- | --- | --- |
| **S1** | Live simulated market | No | Prices move every few seconds. Charts show real price paths. Markets show 24h change, high and low. Resting limits fill when the market crosses them. The frontend polls when no WebSocket is configured. |
| S2 | Durable trading state | No | Orders and fills are journalled in Postgres. The engine replays them on startup. The `Idempotency-Key` on `POST /orders` is honoured. |
| S3 | Demo cNGN faucet deposit | Yes | A setup script creates the demo cNGN mint. `POST /wallet/deposit` mints up to a daily cap into the investor's cNGN account (created idempotently). The UI gets a **Deposit** button and a live balance. |
| S4 | Activation and delegated orders (= U2) | Yes | Wallet activation is explicit. An order is accepted after the investor signs an SPL `Approve` covering its reservation (cNGN for buys, tokens for sells). Buying power and sellable quantity come from on-chain balances and delegations. |
| S5 | On-chain settlement (= U3) | Yes | Each fill settles durably and idempotently. NGX instrument mints are created on first use. House sales deliver from float, then mint; house purchases retire mints, then refill float. Confirmation includes readback. |
| S6 | One trading path and a chain-backed portfolio (= U4) | Yes | The direct-buy endpoints are retired. Holdings and P&L are read from chain balances plus settlement records. Receipts carry explorer links. |

S1 and S2 make the simulator usable locally (matching, prices, charts, history). S3–S5 make money and holdings real on Devnet. Until S4 lands, the engine does not check buying power, and fills move only engine balances.

## S1 — live simulated market (done 2026-10-10)

Implemented as described below.

- **Tests.** `price_feed`, `market_views` and `engine_service` tests cover determinism, bounds, candle sampling, and limit fills on market moves.
- **Frontend polling.** `livePollInterval` in `rabovel-FE/src/lib/api/client.ts` drives polling in real mode.
- **Portfolio source.** Real-mode portfolio summary and holdings now read `/portfolio/*`: engine positions plus on-chain cNGN cash. They used to read `/investor/catalog`.
- **Buying is blocked until S3/S4.** The order form blocks buys when cash is below the order total, and cash is on-chain cNGN. Locally there's no Solana config, so buys stay blocked until S3 (deposit) is configured. The backend itself doesn't check buying power before S4.

### Original S1 scope

- **`PriceFeed`.** Add `price_at(ticker, decimals, unix_secs)`; `latest` becomes `price_at(now)`.
  - `SimulatedPriceFeed` keeps the per-ticker base price. It layers three seeded sine waves (days, hours, minutes) and a small per-tick jitter on top. Ticks are 5 s, prices are rounded to 0.01 NGN, and the range is bounded at about ±15%.
  - Deterministic, so history needs no storage.
- **Candles.** Candles sample the feed inside each bucket for open, high, low and close. Volume comes from engine trades, and trade prices widen the high and low.
- **`/markets`.** Adds 24h change, change %, high, low and volume.
- **Engine tick.**
  - Add `PortfolioTradingApi::reprice_resting()`, which reprices every symbol with resting orders at the current reference price.
  - Bootstrap spawns it every 5 s and publishes the resulting `TradeExecuted` events.
- **Frontend.** Market, book, trade and portfolio queries refetch every 5 s in real mode when no WebSocket URL is set.

## Unresolved decisions

- **Faucet cap and amount**, e.g. ₦1,000,000 per day per wallet, and whether the cap is per wallet or per user.
- **Devnet operations.** Fresh admin, broker and issuer keys (the committed keypairs are exposed), SOL funding for the Admin fee payer, and RPC rate limits. A public RPC may throttle 146 on-demand mint setups.
- **Market hours.** Always open (current), or simulated NGX trading hours.
