# Repository commands

Run from `rabovel-BE/`. Commands below are grounded in Cargo manifests, CI, Docker configuration and existing runbooks; listing a command is not a claim it was executed during documentation setup.

## Toolchain, build and checks

CI and Docker pin Rust 1.98; `solana-adapter` declares edition 2024 and `rust-version = "1.98"`. Other workspace crates use edition 2021. Cargo handles dependency installation during build; no separate install script exists.

```bash
cargo build --workspace
cargo test -p domain --locked
cargo test -p rabovel-api --lib --locked
cargo test -p solana-adapter --all-targets --locked
```

Choose the relevant package/test filter first. Default tests do not require running infrastructure; the validator test is ignored. CI's full checks are:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
docker build -t rabovel/api:ci .
```

Use `cargo fmt -p <package>` to format a changed crate. `--offline` can be added to adapter tests/Clippy when dependencies are already cached, as in the existing ACL runbook.

## Development

Configure environment variables using [README configuration](../README.md#configuration) and [quick start](../README.md#quick-start), preserving existing local secrets. The source binary reads exported process variables, not `.env` automatically. Host-run services require localhost backend URLs; Compose service names apply inside its network.

```bash
docker compose up -d --build
curl -s http://localhost:3000/health
```

For a source-run API, start dependencies with `docker compose up -d postgres redis kafka`, export the required configuration, then run `cargo run -p rabovel-api`. Migrations run on API startup when Postgres is configured. Debug builds expose `/swagger-ui`; release builds do not.

With Kafka running, worker binaries can be run individually:

```bash
RUST_LOG=info KAFKA_BOOTSTRAP_SERVERS=localhost:9092 cargo run -p settlement-worker
RUST_LOG=info KAFKA_BOOTSTRAP_SERVERS=localhost:9092 cargo run -p tokenization-worker
RUST_LOG=info KAFKA_BOOTSTRAP_SERVERS=localhost:9092 cargo run -p reconciliation-worker
```

## Solana integration and local commands

Opt-in local mint integration (starts a validator in a separate terminal and submits local transactions):

```bash
solana-test-validator --ledger /tmp/rabovel-mint-validator
cargo test -p solana-adapter --test local_mint -- --ignored
```

The test uses localhost:8899, requests an airdrop and creates a mint. It does not verify ACL or settlement. Follow [mint setup](../demo/03-create-local-mint.md) for the configured `create_equity_mint` example and [shared lists](../demo/04-shared-lists.md) for `setup_shared_lists`, external program prerequisites and exact arguments. Keep keys outside the repository. The existing [foundation verification guidance](../demo/README.md#local-configuration) excludes service startup/transaction submission from automated foundation verification.

There is no repository Anchor build or custom program build command. The shared-list runbook's `cargo build-sbf` targets separately fetched Gate source, not a crate in this workspace. Per-mint ACL verification commands and limits are in [its runbook](../demo/05-mint-acl-configuration.md#tests-only).
