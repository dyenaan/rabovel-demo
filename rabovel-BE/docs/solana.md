# Rabovel Solana implementation

`crates/solana-adapter` is an off-chain RPC adapter, not an on-chain program. There is no Anchor workspace or Rabovel program source here. It depends on external Token-2022, Token ACL and ABL Gate deployments. When explicitly configured, the gateway invokes it through the durable issuer asset-setup operation; workers are not wired to it.

## Source map and mint setup

- `src/lib.rs` exposes `equity_setup_service` from the existing `EquitySetupService/` directory; its builder file is `TokenMintBuilder.rs`. Preserve these explicit module paths.
- `authority_config/` separates public `AuthConfig`, service-controlled `SignerConfig` sources and loaded `AuthoritySigners`. Hosted API startup decodes Base64 keypair variables directly into validated in-memory signers; local commands retain file support. Resolve authorities from trusted configuration, not request bodies.
- `EquitySetupService/mint_setup.rs` creates a mint and verifies it after confirmation. `token_acl/` holds list builders, saved deployment configuration, ACL handover validation and the resumable per-mint setup service.
- `examples/create_equity_mint.rs` and `examples/setup_shared_lists.rs` are local commands. `tests/local_mint.rs` is an ignored validator integration test.

Mint creation uses System account creation, Token-2022 extension initialization and `InitializeMint2`, then metadata initialization/update instructions. It funds space including metadata TLV growth, signs with required keys selected from Admin, issuer and the saved mint keypair, submits through RPC, confirms and reads back. Existing mint addresses are rejected for creation; uncertain outcomes require inspection before retry.

The resulting mint has zero supply and zero decimals. Extensions are MetadataPointer (pointing to the mint itself), DefaultAccountState (frozen), Pausable, TransferHook (initially no active hook program) and on-mint token metadata. No PermanentDelegate is installed. Creating a mint does not issue inventory.

| Authority | Configured role |
| --- | --- |
| Mint, metadata update, metadata-pointer update | Issuer |
| Initial freeze, pause, transfer-hook update | Rabovel Admin |
| Native freeze after ACL handover | Token ACL MintConfig PDA; Admin remains the configured authority in ACL state |
| Mint creation payer | Admin |

## Accounts, instructions and serialization

`TokenAclBuilder` uses `token-acl-client = 0.3.1` for `CreateConfig` and MintConfig derivation/deserialization. It verifies the Token-2022 owner, frozen default and freeze authority; MintConfig validation checks PDA, owner, discriminator, bump, mint, Admin, Gate and flags. The code expects 100 config bytes because the generated client's `LEN` is zero in this version.

`SharedListsBuilder` derives allow and block list PDAs under `ABL_GATE_PROGRAM_ID` from `["list_config", admin, saved_seed_pubkey]`. The configured seeds are distinct, public deployment identifiers reused across equity mints. Its narrow Gate `CreateList` encoding is discriminator 1, mode byte (allow 0/block 2), then a 32-byte seed. Admin and payer sign; `SharedListsService` submits or verifies existing lists with readback.

`MintAclService::configure` reads and validates program executability, mint, MintConfig, both lists and thaw metadata, then advances only through verified states:

1. ACL `CreateConfig` delegates native freeze authority; verify the resulting config with permissionless operations disabled.
2. Gate `SetupExtraMetas` (discriminator 4) associates both lists; verify owner and exact TLV contents.
3. ACL `TogglePermissionlessInstructions` enables thaw while leaving permissionless freeze disabled; verify the final state.

Admin signs and pays for this setup; issuer and mint keypair are unnecessary. Already-ready state returns no new signatures; valid intermediate states resume. Conflicting state, uncertain submission or failed/non-advancing readback stops execution. The RPC transport requires confirmed or finalized commitment. The API persists the intended deterministic mint address before execution and records verified stage signatures and addresses afterward; uncertain submission becomes `reconciliation_required` and is not automatically retried.

The thaw metadata PDA uses `["thaw_extra_account_metas", mint]` under Gate. Its TLV list uses `CanThawPermissionlessInstruction` and four entries: allow list, allow wallet entry, block list, block wallet entry. Wallet-entry derivation uses `["wallet_entry", list, holder]`; holder bytes come from token-account data offset 32. List account indexes are 6 and 8 in Gate's CPI layout, which includes the permissionless flag account at index 4. This describes ACL/Gate permissionless-thaw resolution, not transfer-hook metadata. The backend constructs setup instructions; external programs perform the on-chain CPIs.

After confirmed setup, the issuer inventory endpoint can add the configured issuer settlement wallet to the allow list, create and permissionlessly thaw its Token-2022 ATA, then mint the asset's full authorized supply to that account. Admin pays for account/list operations and the configured issuer signs `MintTo`. Readback reports supply, inventory balance and readiness; a conflicting non-zero state stops rather than issuing again. This synchronous demo path is not a backing ledger or durable reconciliation worker.

## Existing references and limits

Use the [mint runbook](../demo/03-create-local-mint.md), [shared-list runbook](../demo/04-shared-lists.md), or [per-mint ACL contract and wire-format notes](../demo/05-mint-acl-configuration.md) only for the relevant operation. The latter records Gate encoder provenance and pinned interface versions; per-mint ACL setup has in-memory tests but is explicitly **not verified on-chain**. Executable program accounts alone do not attest compatible binary versions.

The [canonical on-chain design](../demo/ONCHAIN_REFERENCE.md) includes future and historical behavior. Consult selected sections for design intent, using [import scope](../demo/README.md) and current source to establish what exists here. Investor membership and activation, backing records, revocation, settlement authorization, production custody/signing and worker recovery remain pending. An inactive TransferHook extension is not an implemented settlement hook.
