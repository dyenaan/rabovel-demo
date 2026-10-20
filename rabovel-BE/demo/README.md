# Solana adapter runbooks

Active runbooks:

- [Create a local equity mint](03-create-local-mint.md).
- [Create/verify shared lists](04-shared-lists.md).
- [Configure per-mint ACL](05-mint-acl-configuration.md).

The API now invokes setup, inventory and direct-purchase providers. These standalone examples remain useful operator tools; unit tests do not establish live-network compatibility or complete recovery. See [current architecture](../docs/architecture.md), [chain details](../docs/solana.md) and [settlement limits](../docs/settlement-flow.md).

## Local configuration

Keep signing material outside Git. Existing local repository keys were preserved under `~/.config/rabovel/keys` during cleanup; reuse the actual files/configuration rather than creating replacement authorities accidentally. If a destination already existed, the preserved repository file has a `*-from-repo-20261020.json` name.

```bash
mkdir -p ~/.config/rabovel
cp demo/authority-config.example.json ~/.config/rabovel/authorities.json
```

The example resolves relative `keys/*.json` paths against the copied config directory. Adjust issuer aliases/paths to your own configuration. Mint keys and public ACL seed configuration should be retained for uncertain-result inspection/replay. Current API hosting uses validated secret-byte sources rather than these example files.

Automated foundation verification compiles/tests code without starting validators or submitting transactions. Run mutation examples and ignored validator tests only in their documented opt-in environments.

## Historical decisions

[Original mint decision](../docs/history/01-equity-mint.md), [holder eligibility decision](../docs/history/02-holder-eligibility.md) and [import-era slice report](../docs/history/DEMO_SLICES.md) are archived. [ONCHAIN_REFERENCE](ONCHAIN_REFERENCE.md) retains protocol/design context; current source and the operational runbooks determine implemented behavior.
