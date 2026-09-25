# Per-mint ACL configuration

The [canonical reference](ONCHAIN_REFERENCE.md) remains the design authority.
This slice adds `token_acl::MintAclService` to the Solana adapter. It is implemented
and tested without a validator; it has **not** been verified on-chain.

## Service contract

`MintAclService::new(transport).configure(&deployment, mint, &admin).await`
accepts a trusted saved `AclDeploymentConfig`, an existing equity mint and the
configured Admin signer. `RpcClient` implements `MintAclTransport`; tests use an
in-memory implementation. The service uses Admin as fee payer. It does not need
the issuer key or mint keypair.

The service reads the mint, MintConfig, thaw metadata, both shared lists and both
program accounts together, then chooses the next necessary operation:

1. Verify the deployed programs are executable and the existing shared lists
   match the saved Admin, seeds and allow/block modes.
2. If MintConfig is absent, submit ACL CreateConfig to hand over native freeze
   authority. Read back and verify delegation, Admin, Gate and disabled flags.
3. If thaw metadata is absent, submit Gate SetupExtraMetas with **both** shared
   lists. Read back its owner and exact TLV policy, including wallet-entry PDA
   seeds and account-resolution indexes.
4. Only after that verification, submit ACL TogglePermissionlessInstructions
   with freeze disabled and thaw enabled. Read back the full configuration.

The result contains mint, MintConfig, thaw metadata address and the signatures
submitted by this invocation. An already-configured mint returns no new
signatures. Valid intermediate states resume at the next step. Conflicting
metadata or authorities are rejected rather than overwritten. Enabled thaw with
missing/incorrect metadata is an error, not a ready result.

Each submission is confirmed before further reads. The RPC transport requires
confirmed or finalized commitment. An uncertain submission, failed readback or
non-advancing state stops the flow; it is not automatically resubmitted. Errors
retain the uncertain transaction signature when available and all signatures
already confirmed during this invocation.

## Interface provenance

The narrow Gate instruction encoder follows
[`SetupExtraMetas` at c525fa7](https://github.com/solana-foundation/token-acl-gate/blob/c525fa710883a55cedd0cd7bf2a5aef6eac05171/program/src/instructions/setup_extra_metas.rs).
The published Gate client still has incompatible Solana dependency pins.
ACL instructions use `token-acl-client = 0.3.1`; TLV discriminator and serialization
use `token-acl-interface = 0.3.1` and `spl-tlv-account-resolution = 0.11.4`.

The thaw metadata PDA uses `["thaw_extra_account_metas", mint]` under ABL Gate.
Its four entries are allow list, allow wallet-entry lookup, block list and block
wallet-entry lookup. Gate CPI indexes 6 and 8 identify the lists; token-account
data at index 1, byte 32 supplies the holder address. The permissionless flag
account at CPI index 4 is included in those index calculations. This is not
transfer-hook metadata.

## Tests only

```bash
cargo test -p solana-adapter --all-targets --locked --offline
cargo clippy -p solana-adapter --all-targets --locked --offline -- -D warnings
```

The adapter suite passes 31 unit tests (11 added for this slice); its validator test remains ignored.

Tests cover wire format, required signing privileges, both lists and lookup seeds,
configuration validation, step ordering, intermediate-state resumption, ready
no-op, uncertain submissions, stale/failed/corrupt readback, and distinct mint
accounts sharing the same lists. These tests do not execute the deployed programs
or prove Alice/Bob thaw outcomes. No validator or transaction CLI is required.

## Remaining boundary

This service does not create lists, add/remove members, activate holder accounts,
mint inventory or implement settlement. It does not enforce wallet revocation.
Program executability is not binary/version attestation. Trusted configuration
still requires compatible deployments and a correctly selected RPC network.

Signed transactions and operation progress are not persisted here. Before wiring
this service into replaying workers, introduce durable preparation/submission and
reconciliation. A retry after an uncertain result must first reconcile the prior
signature and state. This is especially important before adding issuance or
settlement, where repeating a transaction can repeat an economic effect.
