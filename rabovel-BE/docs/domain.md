# Domain concepts in the current code

## Identity and authorization

`crates/domain/src/auth.rs` owns these rules. An `IdentityProfile` belongs to a backend user; a verified `WalletConnection` records wallet ownership separately. Trust-bearing identity and compliance evidence must be constructed server-side after verification, not accepted as client-provided status flags.

Onboarding is computed as `blocked` for a blocking risk tier, `requires_review` for unassessed risk or missing required checks, and `approved` otherwise. The default policy requires verified email/phone/KYC, accepted terms, MFA enrollment, device trust and a linked wallet. MFA/device-trust endpoints are missing, so normal onboarding cannot currently reach approval. Do not silently relax policy to make a financial path work.

Actions also require wallet ownership and principal permissions; financial actions require the trader role and approved onboarding. Sensitive actions require fresh MFA. Google is the implemented sign-in flow; other enum variants are not proof of implemented authentication providers. Wallet linking supports SIWE/EIP-155 wallets and Phantom Solana wallets. The Solana path issues a session-bound, single-use message challenge and verifies its Ed25519 signature before persisting the holder address. Preparing an authorized live-listing purchase adds that linked wallet to the demo equity ACL and prepares its Token-2022 account; KYC approval alone does not update chain eligibility.

## Orders, portfolios and tokenization

`crates/portfolio-trading/src/lib.rs` defines buy/sell order requests by symbol and string quantity, acknowledgments, order states and portfolio balances. The stub parses quantity as `f64` and rejects parse failures, NaN and nonpositive values; it is not a complete financial validator. It generates process-local `ord_N` identifiers, persists no orders, returns empty portfolios/order lists, and acknowledges cancellation without tracking it. An accepted order is not an executed trade or ownership record.

The gateway's tokenization request contains `asset_ref` and string `amount`. It returns an identifier and emits a request event; status always reports `pending`. Issuer backing evidence is metadata-only and immediately demo-verified; it is not an external attestation or stored document. Live listing requires that simulated verification and confirmed initial inventory. `TradeExecuted`, `PortfolioChanged`, `TokenizationCompleted` and `TokenizationFailed` schemas are contracts, not evidence that the corresponding workers run.

## Equity and holder concepts

The Solana foundation creates zero-supply, zero-decimal equity mints with issuer mint authority and Rabovel Admin controls. See [solana.md](solana.md) for actual authority assignments. Shared allow/block lists and per-mint ACL setup exist; holder membership, activation and revocation workflows remain pending. Backend KYC approval must not be assumed to update chain eligibility automatically.

The live investor catalog maps issuer assets to confirmed mints and reads on-chain inventory and balances. Its direct purchase path atomically transfers cNGN to the configured broker account and equity inventory to the linked Phantom wallet in one partially issuer-signed transaction. It has no durable settlement ledger, idempotency record, matching engine, refunds or automated reconciliation; investor custody, production signing and legal/economic ownership rules remain unresolved.
