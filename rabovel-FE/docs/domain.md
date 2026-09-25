# Frontend domain concepts

These are UI contracts and fixture behavior, not authoritative business rules. Start with the relevant file in `src/types/` or feature-local types; do not assume every enum value represents a working backend capability.

| Concept | Current representation |
| --- | --- |
| Investor | `investor.types.ts` separates account type/status, KYC and eligibility. `AuthUser` separately carries UI role and optional investor ID. |
| Asset | `asset.types.ts` describes issuer, asset class, lifecycle, optional NAV/market price, documents and optional chain/contract metadata. Asset classes include equity, treasury, credit, real estate, infrastructure and commodities. |
| Market/order/trade | Separate contracts in `market.types.ts`, `order.types.ts`, `trade.types.ts`. Market orders, limit and stop-limit are selectable UI values; an order is not a fill or settlement. |
| Portfolio | Holdings, balances, performance and activity from fixture/HTTP adapters under `features/portfolio`. |
| Settlement | Asset/cash legs linked to a trade, lifecycle status, optional chain, custody route, finality policy and transaction hash. See [settlement-flow.md](settlement-flow.md). |
| Primary market | Feature-local subscription/redemption records and reserve coverage. Mock submissions create SUBMITTED records, not issued tokens or executed redemptions. |
| Custody | Wallet address/network/type/status and route metadata. CUSTODIAL, SELF_CUSTODY and MPC are display types, not implemented custody providers. |
| Compliance/reserves/reconciliation | Review queues, restrictions, attestations and mismatch views backed by fixtures; their presence does not establish actual checks or attestations. |

Money and quantities generally travel as decimal strings. `src/lib/formatters.ts` uses Decimal and defaults money display to NGN/en-NG; `components/financial/` reuses these formatters. Some existing validation, charts and balance comparisons still use `Number`, so do not claim arbitrary-precision arithmetic throughout the UI. Display defaults and demo currencies do not define supported settlement assets.

Login/register use mock identities; browser roles are INVESTOR, ADMIN, COMPLIANCE, OPERATIONS, ISSUER and SUPPORT. The admin layout admits the first three back-office roles listed in [architecture.md](architecture.md); this is not server authorization. Onboarding stores self-entered identity, eligibility and custody choices locally and simulates submission. A local upload/eligibility flag is not verified KYC evidence.

The trading form takes eligibility from `mockInvestors`, falling back to its first record, even outside mock API mode. Its balance/market checks are UI feedback, not proof of order admissibility. Stop-limit selection currently has no separate stop-price input; do not invent trigger semantics.

Unresolved: backend investor/role mappings, verified onboarding and wallet ownership, supported assets and jurisdictions, custody/signing responsibilities, precise order/risk/fee rules, reserve evidence, and settlement finality policies. No rewards engine or liquidity-management implementation is established by this frontend.
