---
name: implement-slice
description: Implement one understood Rabovel frontend slice from a concrete task or saved plan with focused verification.
---

# Implement one frontend slice

1. Identify the task and acceptance criteria; read `AGENTS.md`, only relevant docs and the existing diff. Inspect likely affected route/feature/shared files; read the relevant installed Next guide before code changes.
2. Split broad requests into coherent slices. Surface material domain/API assumptions rather than inventing behavior or treating mocks as working integrations.
3. Make the smallest coherent change, preserving component boundaries and unrelated local work. Reuse UI primitives, query keys and financial formatters; avoid opportunistic refactors and architecture redesign.
4. Use `docs/commands.md` for the narrowest applicable lint/type checks and route verification. Cover affected loading/error/empty/success states, roles, mobile layout and keyboard interaction where relevant. Do not claim a test suite exists or equate mock success with backend execution. Fix issues directly caused by the change; report unrelated failures separately.
5. Update only stale documentation sections and record unresolved decisions or follow-up work in the relevant plan/docs.
6. Summarize behavior/files changed, important reasoning, assumptions, checks actually run and results, and remaining work. Link durable context rather than reproducing documents.
