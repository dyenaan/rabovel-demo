# Solana boundary in the frontend

There is **no Solana execution implementation in this folder**: no Rust/Anchor program, Solana SDK dependency, wallet-adapter integration, RPC client, instruction builder, signer, PDA derivation or transaction submission. This document records where chain concepts appear so future sessions do not infer execution from UI labels.

- `src/types/common.types.ts` includes SOLANA among blockchain labels and SPL among token-standard labels; it does not distinguish Token-2022 account extensions.
- `asset.types.ts` optionally exposes blockchain/token standard/contract address; `custody.types.ts` describes wallet addresses and custody routes.
- `features/custody/components/connect-wallet-dialog.tsx` collects address details. `useConnectWallet` in `features/custody/hooks/use-wallets.ts` adds a PENDING_VERIFICATION fixture entry regardless of the mock flag; it performs no proof-of-wallet-ownership or signing.
- Settlement components display optional chain ID, transaction hash and finality labels supplied by API/fixture data. `features/settlements/utils/finality.ts` derives a display state from the settlement status, not from RPC confirmations.
- `features/admin/components/issuance-form.tsx` waits and shows a success toast; it does not create a mint or issue tokens.

The sibling backend contains an independent Token-2022/Token ACL adapter. For a task that actually spans that boundary, consult [backend Solana documentation](../../rabovel-BE/docs/solana.md); do not load backend docs for routine frontend work. That adapter is not connected to these screens or to backend workers.

Unresolved integration includes wallet proof/custody, frontend asset-to-mint mapping, transaction request/response contracts, network selection, permissions, signing ownership, and chain-specific finality. The frontend's generic SAFE/FINALIZED labels are not an agreed Solana commitment policy. Mint creation, eligibility, issuance and settlement must remain distinct operations; no on-chain accounts, CPI relationships or serialization formats can be documented as implemented here.
