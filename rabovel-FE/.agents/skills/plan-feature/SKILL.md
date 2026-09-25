---
name: plan-feature
description: Plan one Rabovel frontend feature or technical slice with a durable implementation handoff before coding.
---

# Plan one frontend slice

1. Identify outcome, affected route/role and scope; read the repository `AGENTS.md` and only the relevant docs. Inspect the smallest likely feature/component/API area, expanding only for specific dependencies.
2. Establish current behavior from source, including whether it is fixture-backed, browser-persisted or a real request. Do not infer backend or Solana capabilities from UI labels. Read the relevant installed Next guide for framework decisions.
3. Identify required changes, likely files, risks and unknown contracts. Prefer one vertical slice; split broader work into slices and detail only the next actionable one. Avoid unrelated redesign and speculative exploration.
4. Save a small plan at the user's requested location or `docs/plans/<slice>.md`, creating that directory only when needed. Include outcome/non-goals, current behavior/source references, implementation steps, acceptance criteria, narrow checks and unresolved decisions. Include relevant loading/error/empty states, roles and responsive behavior without copying architecture docs.
5. Stop before coding unless implementation was explicitly requested; when authorized, continue within scope. Make the saved plan sufficient for a fresh session without this conversation.
