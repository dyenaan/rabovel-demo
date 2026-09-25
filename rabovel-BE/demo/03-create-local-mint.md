# Create the local equity mint

This command uses the agreed mint builder, independently of the older browser issuance flow.
It creates a zero-supply mint, verifies its state, and prints its address and transaction signature.
Rabovel Admin pays rent and fees. The configured issuer controls minting and metadata.
ACL delegation, treasury creation and token issuance are subsequent steps.

Start a validator in another terminal (omit reset to preserve its ledger):

```bash
solana-test-validator --ledger /tmp/rabovel-mint-validator
```

Generate the mint's own keypair once, separately from the authority keypairs:

```bash
(
  umask 077
  solana-keygen new --no-bip39-passphrase --silent \
    --outfile ~/.config/rabovel/keys/dangote-mint.json
)
```

Fund the configured admin on localnet:

```bash
solana airdrop 5 --url http://127.0.0.1:8899 \
  --keypair ~/.config/rabovel/keys/admin.json
```

From the repository root:

```bash
cargo run -p solana-adapter --example create_equity_mint -- \
  ~/.config/rabovel/authorities.json \
  demo-dangote \
  ~/.config/rabovel/keys/dangote-mint.json \
  demo/dangote-metadata.local.json
```

The example deliberately connects only to localhost. The metadata fixture has an explicitly
unresolvable placeholder URI; replace it with your pinned IPFS JSON URI before a presentation.
The command stores the URI; it does not publish or fetch metadata.

Readback verifies Token-2022 ownership, initialized state, zero supply/decimals, all configured
authorities, self-referencing metadata pointer, full on-chain metadata, frozen default,
unpaused state, inactive hook and absence of PermanentDelegate.

Keep using the same mint keypair when inspecting an uncertain result. A rerun rejects an
existing mint address. Submission errors include the signed transaction's signature; a
confirmation/readback error does not prove creation failed. Do not generate a replacement
mint just because RPC timed out. This local command does not yet persist signed transactions
or implement automatic recovery.

Automated local-validator integration test (requires a running validator):

```bash
cargo test -p solana-adapter --test local_mint -- --ignored
```

The test uses temporary in-memory authority keys and local faucet funds.
