---
name: implement-slice
description: Implement one already-understood Rabovel backend slice from a concrete task or scoped plan, with targeted verification and a durable handoff.
---

# Implement one slice

1. Identify the task and acceptance criteria from the request or saved plan; read the repository `AGENTS.md`. If scope is broad, split it into coherent slices instead of expanding into an architecture rewrite.
2. Read only relevant documentation and inspect the files likely involved, including the existing diff so unrelated work is preserved. Expand searches only for specific dependencies.
3. Surface material assumptions; do not invent domain behavior to fill gaps. Make the smallest coherent change that satisfies the slice, preserving existing architecture and avoiding opportunistic refactors.
4. Use `docs/commands.md` for targeted formatting/checks and the narrowest meaningful tests. Fix issues caused by the change; report unrelated failures separately. Broaden checks when the affected boundaries justify it. Distinguish unit verification from infrastructure or on-chain execution.
5. Update only documentation sections made stale by the change. Record outstanding decisions/work in the relevant plan or docs when a future session needs them.
6. Summarize what changed, files changed, important reasoning, assumptions, checks actually run and their results, and remaining work. Reference durable files rather than reproducing their contents.
