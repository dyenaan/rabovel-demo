# 02 — Holder eligibility

> **Superseded decision record.** Use [ONCHAIN_REFERENCE.md](ONCHAIN_REFERENCE.md) for the consolidated current design and implementation status. This file is retained for history.

Agreed policy: 2026-09-22. Implementation pending; remaining configuration choices below are not yet agreed.

- Use Token ACL with the ABL Gate and both allow and block lists. Block-list membership takes precedence over allow-list membership for Gate-approved thawing.
- Share one allow list and one block list across all Rabovel equities within each deployment environment. Configure every equity mint's ABL Gate settings to reference those same lists; each mint retains its own Token ACL MintConfig PDA. New equity setup must include this association.
- Platform-wide revocation discovers and freezes the wallet's relevant token accounts across every Rabovel equity mint. Track completion per account/mint; updating a shared list does not atomically freeze all existing accounts.
- After KYC is completed **with an approved result**, the Rabovel backend adds the user's verified, bound wallet address to the applicable allow list. Completion alone or a client-supplied approval flag is insufficient. Demo KYC is synthetic.
- Rabovel Demo Admin administers both lists. The backend submits authorized, signed list-update transactions; a backend process has no implicit on-chain authority.
- Subsequent failed checks or an administrative restriction decision (for example, fraud watch) can add that wallet to the block list. Sensitive reasons and identity evidence stay off-chain.
- Revocation must also freeze every relevant existing token account for the affected mint(s), including non-ATAs. Blocking alone prevents Gate-approved thawing, not use of an already-thawed account. Confirm and track unfinished freeze operations.
- The mint's native freeze authority is Token ACL's MintConfig PDA. Rabovel Demo Admin remains the ACL administrative authority; do not describe the Gate as overriding every privileged administrative operation.
- Enable permissionless thaw: `enable_permissionless_thaw = true`. Activation requests must pass the configured ABL Gate (allow-listed and not block-listed); no admin signature is required for this Gate-checked path.
- Disable permissionless freeze: `enable_permissionless_freeze = false`. Rabovel Demo Admin authorizes freezes through Token ACL's administrative path. The backend orchestrates and confirms these transactions.
- Explicitly add the demo broker inventory wallet to the shared allow list under Rabovel Demo Admin authorization. Broker security-token accounts start frozen and must pass the same Gate-checked thaw flow before receiving initial issuance. No broker eligibility bypass; block-list precedence applies to the broker too. Repeat registration for each broker wallet used by the demo.
- An external regulatory/government integration is future scope only. Any later delegation needs explicit authorization and a tested program/signing integration; no government contract or approval is assumed.

## Still to agree

- Unblocking/reapproval policy and activation workflow.
- Handling wallet replacement/multiple wallets, and any issuer holding accounts beyond the agreed broker inventory flow.

## Implementation: first slice

The adapter uses the official `token-acl-client = 0.3.1`.
`EquitySetupService::prepare_acl_handover` reads the mint, checks deployed ACL/Gate
programs are executable, and returns instructions plus required signer addresses.
It does not sign or submit them.

`TokenAclBuilder` builds `create_config` and validates readback. The program
creates MintConfig and transfers native freeze authority in the same instruction.
The stored MintConfig `freeze_authority` field is the administrative public key;
the mint's native freeze authority becomes the MintConfig PDA.

Actual implementation seed: `["MINT_CONFIG", mint]`.
The current 100-byte state contains discriminator, bump, permissionless thaw/freeze
flags, mint, administrative freeze authority, and Gate program.

After handover both permissionless flags remain false (`AwaitingGateSetup`).
Next: create the shared allow/block lists, associate them through the Gate's
per-mint thaw account metadata, then enable permissionless thaw. An enabled flag
alone does not prove the shared lists are correctly configured.

Per-mint Gate setup, wallet list updates, permissionless activation, administrative
revocation orchestration, and a runnable ACL handover command remain to build.
No ACL deployment or validator transaction has been run for this slice.

Source reviewed:
[Token ACL implementation](https://github.com/solana-foundation/token-acl/tree/87c5f9a4182357f90955ef1d02909a3433ebbe67)
and [ABL Gate implementation](https://github.com/solana-foundation/token-acl-gate/tree/c525fa710883a55cedd0cd7bf2a5aef6eac05171).
The local deployed binaries must match the client interfaces before integration testing.

### Shared-list creation instructions

`SharedListsBuilder` now returns the two `CreateList` instructions, derived list
addresses, seed identifiers and required signer addresses. It uses Rabovel Admin
as both lists' authority; the fee payer can be Admin or a separate signer.

Supply two distinct public seed identifiers and save them in deployment
configuration. They are not wallet identities or signing keys. Reusing the same
Admin and seeds reproduces the same list addresses for every equity. Creation is
not idempotent: once submitted successfully, reuse those accounts rather than
resubmitting creation.

The builder targets `GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz`.
The published Gate client 0.3.0 pins `solana-instruction = 3.1.0`, which conflicts
with our metadata client's requirement. The small CreateList encoding is therefore
implemented locally against the reviewed Gate commit above: discriminator 1,
mode 0 (Allow) or 2 (Block), followed by the 32-byte seed. PDA seeds are
`["list_config", admin, list_seed]`.

The builder prepares instructions only. Deployment configuration, submission and
readback are now wired through the [shared-list setup command](04-shared-lists.md).
Per-mint Gate association and wallet membership operations remain pending.
