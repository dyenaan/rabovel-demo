# Unified tradable assets: one catalog, matched off-chain, settled on-chain

**Plan — 2026-10-09.** Extends [principal-matching-engine.md](principal-matching-engine.md). It replaces that plan's "Tradable symbols" carry-forward and redefines its slice 3.

## Outcome

Simulated NGX instruments and issuer-minted assets become the same kind of thing: a **listed asset** with a Token-2022 mint, ACL and broker inventory. Every listed asset is:

- shown in one catalog (the Markets and Assets screens read the same list);,
- matched off-chain by the matching engine (house quote plus resting client orders);
- settled on-chain per fill, using authority the investor delegated when placing the order.

## Decisions taken (2026-10-09)

1. **NGX instruments are seeded through the issuer pipeline.** On bootstrap, DANGCEM, GTCO, ZENITHBANK and MTNN become asset records owned by a platform issuer. When chain config is present, they run the same setup → inventory → backing → listing path as any issuer asset. No second code path. *Revised 2026-10-10: all 146 watchlist NGX instruments are catalog entries assumed minted. Their real mints are created on demand in U3; see "U1 status".*
2. **Investor authority comes from an SPL `Approve` signed when the order is placed.** The investor delegates the reserved cNGN (buy) or equity (sell) to the broker. Settlement transfers with that delegation immediately after the fill. The delegation also serves as the pre-trade reservation.
3. **Direct buy goes through the engine.** "Buy" on the Assets page becomes a market order. `/investor/quote` and `/investor/purchase/{prepare,submit}` are retired once U4 lands.

## Current behaviour (source)

- **Engine catalog.** `CatalogReferencePrices` ([price_feed.rs](../../apps/rabovel-api/src/price_feed.rs)) accepts only tickers from `InstrumentProvider`. That provider is `SimulatedInstrumentProvider`, a hard-coded list of 4 NGX tickers ([instrument_provider.rs](../../apps/rabovel-api/src/instrument_provider.rs)). Any other symbol returns `UnknownSymbol`.
- **Engine state.** Engine state is in memory. `EngineTradingConfig.initial_float` comes from static config only. Fills only change engine balances; the settlement worker only logs.
- **Issuer assets.** They are listed via `listing_status = "live"`, a confirmed `AssetSetupOperation` and `issuance_complete` inventory. They appear only in `/investor/catalog` ([lib.rs](../../apps/rabovel-api/src/lib.rs) `investor_catalog_handler`). Trading them is a separate direct DvP: a quote that isn't persisted, an issuer-partially-signed transaction, and the investor signing and submitting.
- **Investor activation.** Allow-list entry, equity ATA creation and Gate thaw run inside `prepare_purchase` ([mint_setup.rs](../../apps/rabovel-api/src/mint_setup.rs)), not as an explicit step.
- **Signers.** `SignerConfig::load(issuer_id)` falls back to the default issuer keypair, so a seeded platform issuer works with the existing `RABOVEL_ISSUER_KEYPAIR_BASE64`. Equity inventory sits in that issuer's ATA, and the issuer holds mint authority.
- **Price feed.** `SimulatedPriceFeed` prices any ticker deterministically. Issuers never supply prices.

## Slices

| # | Slice | Result |
| --- | --- | --- |
| **U1** | One catalog | Seeded NGX and issuer assets are listed together and both match off-chain in the engine. No on-chain settlement yet. |
| U2 | Investor activation and delegated orders | An order is accepted only after the investor's allow-list/ATA activation and an on-chain `Approve` covering the reservation are confirmed. |
| U3 | On-chain fill settlement | The settlement worker settles each `TradeExecuted` durably and idempotently: delegate transfers on the client side, broker/issuer on the house side, mint for `minted` shortfall, readback, then `TradeSettled`. |
| U4 | One trading path | Assets page Buy/Sell becomes engine orders. Direct quote/purchase endpoints are removed. Portfolio is derived from chain balances and settlement records. |

Hedging, reconciliation and liquidity remain slices 4–6 of [principal-matching-engine.md](principal-matching-engine.md).

## U1 status (2026-10-10): implemented, partly

What landed:

- **Ticker rule.** Every tradable ticker follows the NGX watchlist format: `[A-Z][A-Z0-9]{1,11}`, with no hyphens.
  - Enforced by `domain::assets::is_valid_ticker` on the draft ticker and the metadata symbol.
  - Instrument codes may still contain hyphens.
- **One catalog.** `UnifiedInstrumentCatalog` (`instrument_provider.rs`) combines the 146 watchlist NGX tickers with every issuer asset whose status is `minted` and whose listing is `live`.
  - It is the default provider, so the engine (`CatalogReferencePrices` → `find`) accepts issuer tickers as soon as they're listed. There is no registration step.
  - Legacy hyphenated tickers are excluded.
- **Ticker clashes.** Issuer create/update rejects an NGX-listed ticker (`409`). Listing publish rejects an invalid ticker, an NGX ticker, or a ticker another live asset already uses (`409`).
- **New endpoints.** `GET /markets` and `/markets/:market_id` return the frontend `Market` shape inside `{ data }`, with ids `mkt_<market>_<ticker>`, `origin` and the reference price and house bid/ask. The frontend's real mode already called these routes.

Deviations from the steps below:

- **NGX instruments aren't seeded through the issuer pipeline.** They're catalog entries "assumed minted" (`origin: ngx_listed`, `mint_address: null`). With 146 instruments, running mint setup at bootstrap is impractical. U3 creates their mints on demand at first settlement, so steps 2–3 move to U3.
- **The engine float isn't funded from on-chain inventory yet (step 5).** It still comes from `RABOVEL_HOUSE_FLOAT`. Issuer assets without a float fill by minting, up to `max_minted_unbacked`.
- **The watchlist stays star-driven.** New issuer listings appear on Markets, and users star them to add them to the watchlist.

## U1 — one catalog (original plan)

**Non-goals:** on-chain settlement, delegation, persistence of engine state, removing the direct-buy path.

### Steps

1. **Listed-asset catalog.**
   - Add `ListedAssetCatalog`, an `InstrumentProvider` over the repository (`apps/rabovel-api/src/instrument_provider.rs`).
   - An asset is **matchable** when `listing_status == "live"`.
   - Carry `asset_id`, `mint_address: Option<String>` and `settlement_ready: bool` on `Instrument`. `settlement_ready` means the setup is `confirmed` and issuance is complete.
   - Keep the `/instruments` response shape additive.
2. **Seed definitions.**
   - Move the 4 NGX fixtures into seed `NewAssetDraft`s: market `NGX`, decimals 0, sandbox disclosure, settlement currency `CNGN`, and a configured authorized/initial units default.
   - On bootstrap, idempotently create them under a platform issuer user. Reuse the trusted-role path, or a fixed system user id if no issuer email is configured.
   - Approve them and set `listing_status = "live"`.
   - Seeding must be replay-safe: the canonical key already prevents duplicates.
3. **Chain setup for seeds (only when Solana config is present).**
   - After seeding, run the existing setup executor and initial inventory for any seeded asset that isn't yet `settlement_ready`.
   - Failures leave the asset matchable but not `settlement_ready`, and are logged; they don't block startup.
   - Without chain config, seeds are matchable only, so local dev and tests keep working.
4. **Issuer listing feeds the engine.**
   - `publish_asset_listing_handler` already sets `live`. The catalog lookup is live, so no extra registration is needed beyond step 1.
5. **Engine float from on-chain inventory.**
   - Add `PortfolioTradingApi::fund_float(symbol, qty)`, or an `EngineTradingService` method.
   - At bootstrap (after step 3) and on listing publish, fund the float with the inventory readback for `settlement_ready` assets, idempotently. Track funded amounts so a restart or relisting doesn't double-fund.
6. **Symbol identity.**
   - The engine keys by ticker, but tickers are unique only per issuer + market.
   - Reject `publish_asset_listing` when another live asset already uses the same ticker, and return `409`.
7. **Frontend.**
   - Markets ([markets/page.tsx](../../../rabovel-FE/src/app/(dashboard)/markets/page.tsx)) reads `/instruments` in real mode.
   - Assets keeps `/investor/catalog`, but both now show the same assets.
   - Show `settlement_ready` as "on-chain" / "off-chain only".

### Acceptance

- With no chain config, `/instruments` lists the 4 seeded NGX assets. `POST /orders` for `GTCO` fills against the house as today.
- An issuer asset published live (mock executor in tests) appears in `/instruments`. `POST /orders` for its ticker executes instead of returning `UnknownSymbol`. Before listing, the same order is rejected.
- Restarting (re-running seed) creates no duplicate assets and does not double the engine float.
- Publishing a second live asset with an existing live ticker is rejected.
- `/instruments` and `/investor/catalog` list the same asset ids for a live, `settlement_ready` asset.

### Verification

```
docker compose run --rm tests cargo test -p rabovel-api -p portfolio-trading -p identity --locked
```

Seeded-chain setup needs a configured Devnet run, recorded per [DEMO_SLICES.md](../../demo/DEMO_SLICES.md) rules; mock tests are not chain evidence.

## Notes for U2–U3 (not detailed yet)

- **Reservation size.** For a market buy the price is unknown when the order is placed. Reserve `qty × house ask × (1 + slippage cap) + fee`, and reject a fill that would exceed the delegated amount. A limit buy reserves `qty × limit + fee`. A sell reserves `qty`.
- **Revoking unused delegation.** Revoke or reduce the delegation on cancel, and when the order fills for less than reserved. The investor can also revoke on-chain at any time, so settlement must re-check `delegated_amount` and treat a shortfall as a failed fill that needs an engine unwind.
- **Token-2022 transfers.** Equity transfers with extra metas (`create_transfer_checked_instruction_with_extra_metas`) work with a delegate authority. Verify against Token ACL and the inactive transfer hook on a local validator.
- **Client-to-client fills.** Both legs use delegations, so the broker can submit one transaction alone.
- **Before submitting.** Record each settlement durably (fill sequence → transaction → signature → status) before submitting. Address the [settlement-flow.md](../settlement-flow.md#unresolved-integration-boundaries) replay gaps first.

## Unresolved decisions

- **Seed sizes.** Authorized and initial units for seeded NGX assets, and whether their backing evidence is a fixed sandbox statement.
- **KYC gate.** Whether investor activation (U2) requires an approved KYC verdict in demo mode. `for_demo()` currently sets `require_kyc: false`.
- **Engine durability.** An in-memory engine plus on-chain settlement means a restart loses resting orders whose delegations are still live. U2 needs an order journal or a revoke-on-restart sweep.
- **Slippage cap** for market-order reservations, and whether it is configurable per asset.
