# Mint and ACL setup slice

## Outcome

An issuer can start setup for an `approved_for_setup` asset and see durable progress through metadata readiness, zero-supply Token-2022 mint creation, ACL handover/list association, and verified completion. A completed operation records the mint, ACL accounts, transaction signatures and readback evidence. It does not issue inventory or list the asset for investors.

## Current behavior

- Asset drafts advance to `approved_for_setup`; durable setup operations, mint fields and issuer start/status endpoints are implemented.
- `solana-adapter` can create and read back the selected zero-supply mint and can resumably configure per-mint ACL state.
- Shared allow/block list setup exists as an adapter command/service, but no backend deployment record proves that the configured lists exist for the selected network.
- The configured API executor invokes the adapter synchronously and persists final verified evidence. Worker orchestration remains future hardening.
- Draft metadata is stored, issuer-owned draft images use signed direct uploads, and approval publishes canonical JSON to Supabase Storage before setting `metadata_uri`.

## Next implementation boundary

1. **Implemented:** approval generates validated JSON and uploads it to the public Supabase Storage `metadata` bucket at `assets/{asset_id}/metadata.json`; the URI is backend-owned and approval fails closed if publishing fails.
2. Add network-scoped trusted deployment configuration: RPC/network identity, Admin and issuer signer resolution, Token ACL/Gate program IDs, and saved shared-list addresses/seeds. Never accept keys, signer paths or authority roles from the HTTP request.
3. Add a durable setup operation with stages such as `queued`, `metadata_ready`, `mint_submitted`, `mint_verified`, `acl_handover_verified`, `lists_associated_verified`, `thaw_enabled_verified`, `confirmed`, `reconciliation_required`, and `failed`.
4. Save the intended mint address and signed transaction/signature before broadcast. On uncertain submission, stop in `reconciliation_required`; inspect the known signature and accounts before any retry.
5. Introduce a narrow setup-executor boundary. API/domain tests use a mock executor, while configured runtime execution calls `solana-adapter`. Do not use mock execution to claim an on-chain mint in the demo UI.
6. Expose issuer endpoints to start an idempotent setup operation and read its status. Only the owning approved issuer may start/read it. Repeated starts return the existing operation.
7. Update the issuer lifecycle UI from `approved_for_setup` to show real operation stages, signatures and addresses. Advance to “Inventory” only after confirmed readback; a submitted signature is not confirmation.
8. Wire the tokenization worker only after the durable operation contract exists. Worker replay must resume/reconcile the same operation rather than generate a second mint.

## Non-goals

- Payment-mint setup, backing allocation, inventory issuance, broker ATA activation, investor eligibility, trading or listing.
- Production signer custody or mainnet deployment.
- Claiming the current ACL adapter has been proven on Devnet; that requires a recorded run.

## Acceptance

- Only an owning issuer with an `approved_for_setup` asset can start setup.
- A setup cannot start without hosted metadata and verified shared-list deployment configuration.
- Mint readback proves zero decimals, zero supply, selected extensions, metadata, and trusted authorities.
- ACL readback proves MintConfig handover, exact shared-list association, permissionless thaw enabled and permissionless freeze disabled.
- Refresh shows the same operation and evidence; repeated requests do not create another mint or replay confirmed mutations.
- Failure and uncertain submission remain visible and do not falsely advance the asset lifecycle.

## Verification

```bash
cargo test -p domain --locked
cargo test -p rabovel-api --lib --locked
cargo test -p solana-adapter --all-targets --locked
cargo test -p solana-adapter --test local_mint -- --ignored
```

The ignored validator test is opt-in and currently covers mint creation rather than the complete ACL path. Record a Devnet run separately before presenting the full setup as chain-proven.

## Storage decision

- Use one public Supabase Storage bucket named `metadata`.
- The backend owns metadata JSON generation and upload. Issuers edit structured fields and see a live JSON preview; they do not upload arbitrary JSON.
- Store metadata at `assets/{asset_id}/metadata.json`. Publishing is allowed only after demo approval, and the stored public URL becomes the mint metadata URI.
- Images use `assets/{asset_id}/images/{random_object_id}.{extension}` in the same bucket. After the first draft save supplies an asset ID, the backend issues a short-lived signed upload URL; the browser uploads directly to Storage and then confirms the object key with the backend. Image upload is implemented for editable drafts.
- Before signing an image upload, validate an allow-list of MIME types (`image/png`, `image/jpeg`, `image/webp`), a small demo size limit, ownership and `draft` status. On confirmation, verify the object exists and persist the backend-derived public URL. Orphan cleanup is a separate maintenance task.

## Decisions before live execution

- Target network for the first recorded run (local validator where supported, then Devnet).
- How demo signer material is injected into the worker environment. Keep it outside the repository and HTTP/event payloads.
- Whether shared lists are provisioned by an operations command before the demo or by a separate guarded deployment operation. They should not be recreated per asset.
