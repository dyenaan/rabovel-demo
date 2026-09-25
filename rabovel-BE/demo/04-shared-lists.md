# Shared list setup

The two lists belong to the deployment, not to an individual equity. The local
command loads only Rabovel Admin's keypair; no issuer keypair is needed.

## Save deployment configuration

From the repository root:

```bash
cargo run -p solana-adapter --example setup_shared_lists -- \
  init ~/.config/rabovel/authorities.json ~/.config/rabovel/acl-local.json
```

This generates two public seed identifiers and saves them with the Admin address,
environment and supported Gate program address. It refuses to overwrite an
existing file. Keep this file and reuse it across equities and retries.

## Local Gate prerequisite

The validator must have the ABL Gate executable at
`GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz`. A plain validator does not include it.

For a reproducible setup, build the reviewed source and load its binary at genesis.
Run these commands yourself; they have not been executed as part of this slice:

```bash
git clone https://github.com/solana-foundation/token-acl-gate.git /tmp/rabovel-abl-source
git -C /tmp/rabovel-abl-source checkout c525fa710883a55cedd0cd7bf2a5aef6eac05171
cargo build-sbf --manifest-path /tmp/rabovel-abl-source/program/Cargo.toml \
  --sbf-out-dir /tmp/rabovel-abl-program
```

Stop the current validator before starting another on port 8899:

```bash
solana-test-validator --ledger /tmp/rabovel-abl-ledger \
  --bpf-program GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz \
  /tmp/rabovel-abl-program/token_acl_gate_program.so
```

Use a fresh ledger path for the first load: the validator ignores genesis program
flags if that ledger already exists. This is a separate chain from your previous
ledger; previous mints and balances are not present. The old ledger is preserved.
Token ACL deployment is not needed to create the two lists, but is needed for
the subsequent per-mint handover.

Fund Admin on this ledger:

```bash
solana airdrop 5 --url http://127.0.0.1:8899 \
  --keypair ~/.config/rabovel/keys/admin.json
```

## Create or verify the lists

```bash
cargo run -p solana-adapter --example setup_shared_lists -- \
  create ~/.config/rabovel/authorities.json ~/.config/rabovel/acl-local.json
```

The command connects only to localhost, checks that Gate is executable, and:
- Creates both absent lists in one transaction signed and paid by Admin.
- Verifies existing lists without submitting another transaction.
- Rejects a partial pair or mismatched owner, layout, authority, seed or mode.
- Prints derived addresses, the creation signature when applicable, and wallet counts.

A submission/confirmation failure includes the transaction signature. Keep the
same deployment file and rerun to inspect the same addresses; do not regenerate
seeds as a response to an RPC timeout. Signed transactions are not yet persisted
for automatic recovery.

This completes list creation only. The [per-mint ACL service](05-mint-acl-configuration.md) associates these lists with an equity and enables permissionless thaw after readback; that service is unit-tested but has not been verified on-chain.
