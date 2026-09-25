# Blockchain foundation

[ONCHAIN_REFERENCE.md](ONCHAIN_REFERENCE.md) is the canonical design. This import preserves the selected Token-2022 / Token ACL / ABL Gate foundation, not the old browser demo architecture.

For the full issuer-to-investor implementation order and the next scoped handoff, see [DEMO_SLICES.md](DEMO_SLICES.md).

## Imported

- `crates/solana-adapter`: mint builder and readback, authority/signer configuration, ACL handover preparation/config verification, shared-list builders, saved deployment configuration and local list submission/readback.
- Existing unit tests, the opt-in local-validator mint test, and both command examples.
- [Mint runbook](03-create-local-mint.md), [shared-list runbook](04-shared-lists.md), public example configuration and metadata fixture.

The adapter has no path or runtime dependency on rabo-chain. The existing BE workspace wildcard includes it. No worker consumes it yet. No private keys or ledger state were copied.

## Deliberately excluded

The legacy eligibility-hook program, PermanentDelegate issuance/recovery flow, settlement adapter, old API/authentication, database, frontend and their scripts were not ported. The old settlement-domain/service crates were not needed by this foundation. Historical implementation claims in the reference do not mean those components exist here.

## Local configuration

Run commands yourself from the rabovel-BE root. Do not start services or submit transactions as part of automated foundation verification.

Create local Admin, broker and demo-issuer keypairs outside the repository if you do not already have them:

```bash
mkdir -p ~/.config/rabovel/keys
chmod 700 ~/.config/rabovel/keys
(
  umask 077
  solana-keygen new --no-bip39-passphrase --silent --outfile ~/.config/rabovel/keys/admin.json
  solana-keygen new --no-bip39-passphrase --silent --outfile ~/.config/rabovel/keys/broker.json
  solana-keygen new --no-bip39-passphrase --silent --outfile ~/.config/rabovel/keys/dangote-issuer.json
)
cp demo/authority-config.example.json ~/.config/rabovel/authorities.json
```

Edit the copied configuration to use your actual absolute keypair paths; remove unused issuer entries or configure their keys. Reuse existing keys/configuration when available. Follow the two runbooks for validator prerequisites, funding and transaction commands. The metadata URI is an invalid placeholder, not published IPFS content.

## Build toolchain

CI and the gateway Docker builder now use Rust 1.98, matching the toolchain used to verify the combined workspace. The imported adapter uses edition 2024; existing BE crates retain edition 2021. The combined Cargo.lock includes the Solana dependency graph, with token-acl-client pinned to 0.3.1.

## Verification without a validator

```bash
cargo test -p solana-adapter --all-targets --locked
cargo clippy -p solana-adapter --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

Import verification: all 45 tests across the workspace excluding `order-gateway` passed (including the 20 selected adapter tests), and both examples compiled. Clippy with warnings denied passed for the same workspace subset. Formatting and documentation-link checks passed. Full workspace tests are blocked here by the existing Swagger UI build-time GitHub download; network access was declined. The legacy adapter test count in the reference describes the old repository, not this selected subset.

The ignored `local_mint` test submits transactions only when explicitly selected using the mint runbook. Unit tests and compilation do not establish on-chain verification.

## Next integration boundary

[Per-mint ACL setup](05-mint-acl-configuration.md) now implements handover, Gate/list association and enabling permissionless thaw, with in-memory tests. It has not been verified on-chain. Membership updates, broker/holder activation, backed issuance, block-and-freeze revocation and future settlement authorization remain pending. Durable operation storage and recovery must precede wiring transaction side effects to replaying Kafka consumers. Resolve authorities from trusted backend configuration and reuse BE identities.

Before deleting the old repository, preserve any unrelated work/history you still want. This import replaces only the foundation selected by the canonical reference, not every legacy capability.
