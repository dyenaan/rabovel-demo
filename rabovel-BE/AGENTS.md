# Working in rabovel-BE

Rabovel is a Rust brokerage backend for tokenized assets. The API implements identity, wallet linking and onboarding; trading and downstream workers are currently stubs. A separate Solana adapter implements Token-2022 mint and Token ACL setup, without worker integration.

- Runnable processes live under `apps/`; reusable domain and adapter code lives under `crates/`. Event contracts live in `events`, while Kafka transport lives in `infrastructure::messaging`.
- Inspect the smallest relevant source area with targeted searches; expand only when evidence requires it. Source code is the implementation authority.
- Read only the documentation needed for the task, never all docs by default:
  - Component boundaries: [docs/architecture.md](docs/architecture.md).
  - Identity, orders and asset concepts: [docs/domain.md](docs/domain.md).
  - Mint, authorities and ACL work: [docs/solana.md](docs/solana.md).
  - Events, workers and order-to-chain lifecycle: [docs/settlement-flow.md](docs/settlement-flow.md).
  - Build, dev and verification: [docs/commands.md](docs/commands.md).
  - Detailed API/configuration: relevant sections of [README.md](README.md); chain runbooks are linked from the Solana doc.
- Prefer one small vertical slice per implementation session. Do not redesign unrelated architecture or perform opportunistic cleanup. Surface assumptions and unresolved domain behavior explicitly.
- Run the narrowest meaningful tests first. Do not edit generated, vendor or build output (including `target/`) unless necessary; change the lockfile only when dependency work requires it. Keep secrets and private key material out of documentation.
- Workflows live in `.agents/skills/`: `plan-feature` for a durable scoped plan, `implement-slice` for implementation, `verify-change` for diff review/testing. Load only the applicable skill.
- Leave durable task context in the relevant code/docs. Update the smallest stale section and reference it in the handoff instead of repeating large documents in chat. Add skills only for justified recurring workflows.
