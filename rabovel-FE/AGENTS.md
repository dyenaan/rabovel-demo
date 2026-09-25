# Working in rabovel-FE

Rabovel's frontend presents tokenized-asset investing, trading and operations workflows. It is a Next.js App Router application with mock-backed data and partial HTTP/WebSocket adapters. Screens and status types do not establish working backend or blockchain capabilities.

- Keep routes/layouts in `src/app`, feature components/hooks/API functions in `src/features`, shared UI in `src/components`, contracts in `src/types`, and browser state in `src/stores`. `@/*` resolves to `src/*`.
- Inspect the smallest relevant source area using targeted searches; expand only when evidence requires it. Source is the authority for implemented behavior.
- Read only relevant docs, never every document at task start: [architecture](docs/architecture.md) for component/data boundaries; [domain](docs/domain.md) for concepts; [Solana](docs/solana.md) for chain-facing limits; [settlement flow](docs/settlement-flow.md) for orders/statuses; [commands](docs/commands.md) for setup/checks.
- Prefer one small vertical slice per implementation session. Preserve existing architecture, avoid unrelated refactors, and surface assumptions rather than inventing business rules. UI guards and mock eligibility are not backend authorization.
- Run the narrowest relevant checks first. Reuse existing UI primitives and financial formatters; keep financial contract values as decimal strings and use Decimal for arithmetic.
- Do not edit `node_modules/`, `.next/`, generated `next-env.d.ts`, vendor or build output unless necessary. Change lockfiles only for dependency work; preserve unrelated local changes. Never document private environment values.
- Use only the applicable workflow in `.agents/skills/`: `plan-feature`, `implement-slice`, or `verify-change`. Save scoped plans when needed; leave durable context in code/docs and update only stale sections. Reference documents instead of repeating them in chat; add skills only for justified recurring work.

<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->
