# Agent pipeline — ticket to production

A low-cost, human-gated agent loop for this repo. It is designed so the AI
steps ride subscriptions that are already paid for (ChatGPT/Codex, Muse,
OpenCode) instead of per-token API billing. Marginal cost target: **$0**
beyond those subscriptions. GitHub Actions is free for public repos.

## The loop

```
ticket ──► triage ──► planner ──► builder ──► tests (CI) ──► reviewer AI
  ▲                                                              │
  │                                                              ▼
monitoring ◄── production ◄── [HUMAN GATE] ◄── staging ◄── PR + rubric pass
```

1. **Ticket.** Created from the *AI Build Ticket* issue template
   (`.github/ISSUE_TEMPLATE/ai-build.yml`). Spec first: summary, acceptance
   criteria, tests/evals, out-of-scope, risk. No spec, no run.
2. **Triage.** Label `ai-build`. The `AI Build Triage` workflow validates the
   spec and posts the hand-off pack on the issue.
3. **Planner.** Breaks the ticket into one small vertical slice (repo rule).
   Bigger than one slice → split into sub-issues and stop. Planner can be
   Muse in chat. Uses the `plan-feature` skill in the target area.
4. **Builder.** Codex cloud (assign the issue / `@codex`) or OpenCode on the
   subscription login. Follows `AGENTS.md` in `rabovel-BE/` or `rabovel-FE/`
   and the `implement-slice` skill. Opens a PR; never pushes to `main`.
5. **Tests.** Existing CI is the gate: BE = fmt, clippy `-D warnings`,
   `cargo test --workspace`; FE = node tests under `rabovel-FE/tests/`.
   Red CI = builder iterates or the ticket goes back, capped at 3 attempts.
6. **Reviewer AI (second AI).** A *different* model than the builder
   (builder = Codex → reviewer = Muse/OpenCode-other, or vice versa).
   Scores the PR against [REVIEW_RUBRIC.md](REVIEW_RUBRIC.md), including
   scripted product-flow evals where they exist, and posts the scored
   review as a PR comment. Uses the `verify-change` skill.
7. **PR + human review.** Green CI + passing rubric = merge-ready. A human
   merges. Agents never merge their own PRs.
8. **Staging → production.** Merge deploys to staging behind a feature
   flag; smoke the flows from the ticket's acceptance criteria.
   **Production promotion is a human decision.** `high` risk tickets
   (auth, money movement, chain) never auto-promote, ever.
9. **Monitoring → tickets.** Crash logs, alerts and perf regressions are
   filed as new AI Build Tickets with logs attached, closing the loop:
   caught issue → ticket → fix → improvement.

## Cost control

- The three AI steps (plan, build, review) run on subscription quotas,
  not API tokens. The binding constraint is **quota, not dollars** —
  see [QUOTA.md](QUOTA.md) for reset times and the parking policy.
- Optional API-key fallback exists for mid-sprint quota exhaustion and is
  **off by default**. Turning it on requires editing QUOTA.md in the same
  PR so the spend is visible.
- One ticket in flight per area (BE/FE) at a time. Parallel agent runs are
  how quotas and review attention both get burned.

## What this is not

- Not full autopilot to production. Idea → prototype is safe to automate;
  idea → production without a human gate is where silent wrong behaviour
  ships. The human gates are the design, not a temporary limitation.
- Not a replacement for tests. The pipeline is exactly as trustworthy as
  the test suite + rubric. Weak tests → weak automation, regardless of
  which model builds.
