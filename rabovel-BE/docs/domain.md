# Current domain behavior

## Identity and compliance

Identity owns account/profile/wallet models and `access/policy.rs`. Password signup selects exactly one public investor/issuer role; permissions and roles remain server-controlled. Sessions resolve stored user IDs and financial checks read current principals. Compliance and wallet linking update profiles through a port that cannot replace roles or permissions.

Onboarding computes blocked/requires-review/approved from risk, verification, terms, device/MFA and wallet state. Strict policy lacks public MFA/device enrollment paths; explicit demo policy relaxes those requirements while preserving ownership, roles and permissions. Sensitive authorization still requires fresh MFA. Wallet linking verifies session-bound one-use SIWE or Solana/Phantom proofs. Historical Google identity discriminants remain readable; Google login is retired.

KYC uses a signed mock provider. Issuer organization/backing approval is simulated, and backing documents are metadata rather than independent attestations. KYC approval alone does not update on-chain eligibility.

## Assets and execution

Issuer owns drafts, media/metadata, review/backing, mint setup, inventory and listing lifecycle. Repositories retain canonical uniqueness, owner/status guards and one setup operation per asset. Inventory completion depends on total authorized supply rather than remaining issuer balance. Catalog combines simulated NGX fixtures with eligible published issuer records.

Trading parses exact whole token quantities and fixed-point payment prices. One process-local engine prioritizes client liquidity and house quotes, tracks float/mint inventory and produces executions. Orders lack buying-power/token-balance checks and restart recovery. A matched fill is not on-chain settlement. The direct-purchase flow independently builds/submits cNGN-for-equity transactions; preparation itself can add eligibility/create/thaw accounts.

Portfolio combines positive simulated positions and observed wallet holdings without duplicating cash/asset rows. Wallet cost and historical performance are estimates, not a reconstructed acquisition/settlement ledger. Activity is order-derived. Legacy tokenization request/status stubs were retired; event schemas remain compatibility contracts.

## Sources and future work

See [architecture](architecture.md), [current flows](refactor/current-map.md), [retirements](refactor/retirements.md), [chain behavior](solana.md) and [execution/settlement boundaries](settlement-flow.md). Durable intent, reservations, outbox/inbox, finality/recovery and reconciliation remain the explicit upgrade track in the refactor plan.
