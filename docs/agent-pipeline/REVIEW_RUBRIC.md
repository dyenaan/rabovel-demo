# Reviewer rubric — the second AI

The reviewer must be a **different model** from the builder. It scores the
PR against this rubric and posts the completed scorecard as a PR comment.
A PR is merge-ready only at **≥ 80/100 with no zero in sections 1–3**.

Reviewer instructions: read the linked issue first, then the diff, then
run (do not just read) the checks in section 4. Try to break the change —
adversarial beats agreeable. Score what the code does, not what the PR
description claims.

## Scorecard

### 1. Acceptance criteria — 0–30
- Every criterion in the ticket is demonstrably met (cite file/line or test).
- No criterion silently reinterpreted or partially met.
- Anything unmet is listed explicitly, not smoothed over.

### 2. Scope discipline — 0–20
- Diff touches only what the ticket requires (repo rule: one vertical
  slice, no opportunistic cleanup, no unrelated refactors).
- `AGENTS.md` conventions for the area followed (layout, formatters,
  decimal-string money handling in FE, no secrets/key material anywhere).

### 3. Tests — 0–25
- Existing suite green (BE: fmt/clippy/`cargo test`; FE: node tests).
- New behaviour has new tests, and they actually exercise the acceptance
  criteria — not snapshot filler.
- Tests fail when the fix is reverted (spot-check at least one).

### 4. Behaviour under attack — 0–15
- Run the scripted product flows / evals named in the ticket.
- Probe edge cases the ticket implies: empty states, invalid input,
  permission boundaries (UI guards are not backend authorization),
  duplicate/concurrent actions.
- Report anything that behaves surprisingly, even if out of scope.

### 5. Risk & rollout — 0–10
- Risk label in the ticket matches the diff's actual blast radius.
- Feature flag present for user-facing changes; rollback is one step.
- Migrations/config changes are called out with their failure mode.

## Output format (post verbatim structure)

```
## Reviewer scorecard
Ticket: #<n> · Builder model: <x> · Reviewer model: <y>
1. Acceptance criteria: __/30 — <evidence>
2. Scope discipline: __/20 — <evidence>
3. Tests: __/25 — <evidence>
4. Behaviour under attack: __/15 — <what was tried, what happened>
5. Risk & rollout: __/10 — <notes>
Total: __/100 · Verdict: merge-ready / changes required
Blocking issues: <list, or "none">
```
