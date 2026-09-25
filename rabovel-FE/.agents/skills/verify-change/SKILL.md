---
name: verify-change
description: Review and verify an existing Rabovel frontend change for behavior, regressions, UI quality and integration correctness.
---

# Verify a frontend change

1. Inspect the diff and working-tree status first; identify intended behavior and review scope. Read `AGENTS.md` and relevant docs only as needed. Use the installed Next guide when framework behavior matters.
2. Trace affected routes/components/hooks/adapters through the smallest relevant source area. Check correctness, error/empty/loading states, form validation, decimal handling, cache invalidation, hydration, effect/subscription cleanup and role visibility where affected.
3. Check responsive layout, keyboard/focus behavior and accessible labels for changed UI. Browser guards/local roles are not authorization; confirm mock data is not mistaken for verified identity, eligibility or settlement.
4. For integration changes, examine actual payload/response contracts, auth, idempotency/retry behavior and live/mock differences. For chain-facing changes, distinguish displayed status from verified finality; examine network, address, signing/authority, amount/decimal and transaction recovery boundaries only where implemented or changed. Rust/PDA/CPI details belong to a backend review when that source is in scope.
5. Run the narrowest meaningful lint/type checks and route interactions from `docs/commands.md`; broaden based on risk. Report unavailable checks and distinguish source review, mock UI verification and real backend/on-chain execution.
6. Report findings with file/line evidence, impact and remedy: **BLOCKING** for correctness/security/acceptance failures, **non-blocking** for lesser issues, and **optional improvements** separately. State when no issues were found and what remains unverified. Do not rewrite working code for style or silently implement fixes during review-only work.
