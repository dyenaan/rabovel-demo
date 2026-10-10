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

To test in Docker on the pinned toolchain, use the on-demand `tests` compose service (profile `test`). It defaults to `matching-engine` and accepts any cargo command:

```bash
docker compose run --rm tests
docker compose run --rm tests cargo test --workspace --locked
```

Use `cargo fmt -p <package>` to format a changed crate. `--offline` can be added to adapter tests/Clippy when dependencies are already cached, as in the existing ACL runbook.

## Development

Configure the git-ignored `.env` using [Run locally](../README.md#run-locally), preserving existing local secrets. Use localhost URLs for Postgres and Redis, leave `KAFKA_BOOTSTRAP_SERVERS` unset, and match `RABOVEL_PUBLIC_ORIGIN` to the local frontend origin. The source binary does not load `.env` itself; Cargo Watch loads it for the child process.

```bash
docker compose up -d --wait
cargo watch --env-file .env -x 'run -p rabovel-api --locked'
```

In another terminal:

```bash
curl -f http://localhost:8080/health
```

Default Compose startup runs only Postgres and Redis, bound to localhost. Redis has a 25 MB memory limit and `noeviction`; the test runner remains opt-in. The API runs in the terminal and migrations run at startup. Debug builds expose `/swagger-ui`; release builds do not. Point the frontend at `http://localhost:8080` and disable its mock API mode.

If `cargo watch` is missing, install it with `cargo install cargo-watch --locked`. Restart Cargo Watch after changing `.env`. If a parent shell exports `KAFKA_BOOTSTRAP_SERVERS`, unset it there too. To retire containers left from the previous stack without deleting named volumes, run `docker compose up -d --wait --remove-orphans` once. Do not use `down -v` to preserve data.

Kafka is not provisioned by this Compose file. If you separately provision a broker, the optional worker shells can be run individually:

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
