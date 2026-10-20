# Working in rabovel-BE

Rabovel is a Rust brokerage backend for tokenized assets. The API implements identity, wallet linking, onboarding, issuer issuance and direct Solana purchases. Orders match in memory; downstream workers only log events. The Solana adapter implements Token-2022 mint and Token ACL setup, without worker integration.

- Runnable processes live under `apps/`; reusable domain and adapter code lives under `crates/`. Event contracts live in `events`, while Kafka transport lives in `infrastructure::messaging`.
- Organize feature → cohesive subfeature → responsibility. Keep repositories/providers with their owning feature, use explicit routes/handlers/dto/usecases/ports names, and create only modules needed by actual behavior. Keep one engine instance for orders, repricing, markets and portfolio. Event delivery remains best-effort until the separate durability upgrade.
- Inspect the smallest relevant source area with targeted searches; expand only when evidence requires it. Source code is the implementation authority.
- Read only the documentation needed for the task, never all docs by default:
  - Component boundaries: [docs/architecture.md](docs/architecture.md).
  - Identity, orders and asset concepts: [docs/domain.md](docs/domain.md).
  - Mint, authorities and ACL work: [docs/solana.md](docs/solana.md).
  - Events, workers and order-to-chain lifecycle: [docs/settlement-flow.md](docs/settlement-flow.md).
  - Build, dev and verification: [docs/commands.md](docs/commands.md).
  - Detailed API/configuration: relevant sections of [README.md](README.md); chain runbooks are linked from the Solana doc.
- The authorized whole-backend refactor follows `docs/plans/feature-architecture-refactor.md`; `docs/refactor/flow-map.md` records its source baseline. Use coherent build/test checkpoints while carrying the full campaign to completion. Keep structural moves distinct from behavioral upgrades and preserve unrelated changes.
- The Git root in this checkout is the parent `rabovel-demo`; effective backend CI lives there. Run `python3 scripts/check_architecture.py` for boundary changes.
- Run the narrowest meaningful tests first. Do not edit generated, vendor or build output (including `target/`) unless necessary; change the lockfile only when dependency work requires it. Keep secrets and private key material out of documentation.
- Workflows live in `.agents/skills/`: `plan-feature` for a durable scoped plan, `implement-slice` for implementation, `verify-change` for diff review/testing. Load only the applicable skill.
- Leave durable task context in the relevant code/docs. Update the smallest stale section and reference it in the handoff instead of repeating large documents in chat. Add skills only for justified recurring workflows.
