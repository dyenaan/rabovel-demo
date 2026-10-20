---
name: plan-feature
description: Plan one Rabovel backend feature or technical slice and leave a durable implementation handoff. Use for scoped planning before coding.
---

# Plan one slice

1. Identify the requested outcome and boundaries; read the repository `AGENTS.md`.
2. Follow only documentation routes relevant to the outcome. Search the smallest likely source area and expand only to resolve a concrete dependency or unknown; do not explore the whole repository by default.
3. Establish existing behavior from source. Separate implemented behavior, proposed changes and unresolved domain decisions, especially worker stubs versus chain execution.
4. Map all affected flows and agree target ownership, contract changes, risks and assumptions before implementation. Use verifiable checkpoints while retaining complete destinations and completion criteria for an explicitly requested repository-wide refactor.
5. Write a concise durable plan at a user-specified location or `docs/plans/<slice>.md` (create that directory only when needed). Include outcome/non-goals, source references, current behavior, small implementation steps, acceptance criteria, narrow verification commands and unresolved decisions. Avoid copying architecture docs.
6. Stop before coding unless implementation was explicitly requested; when authorized, continue within that scope. The plan must let a fresh session implement the slice without needing this conversation.
