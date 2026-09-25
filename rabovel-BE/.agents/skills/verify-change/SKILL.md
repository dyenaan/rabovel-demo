---
name: verify-change
description: Review and test an existing Rabovel backend change for correctness and regressions, including Rust and Solana concerns when applicable.
---

# Verify an existing change

1. Inspect the diff and working-tree status first; establish the intended behavior and review boundary. Read the repository `AGENTS.md`, then relevant docs only when needed to resolve a contract or assumption.
2. Trace changed behavior through the smallest relevant set of callers/tests. Check correctness, failure handling and regressions against acceptance criteria. Do not rewrite working code for style preference.
3. For Rust changes, check error propagation, panics in request paths, async/concurrency behavior, numeric conversions and arithmetic where affected. For event/worker changes, examine delivery loss, replay and economic duplicate effects.
4. For Solana changes, check applicable account ownership/validation, signer and authority privileges, PDA seeds/program IDs, token program/mint/account relationships, checked amounts/decimals, serialization/TLV layouts, CPI account ordering, transaction signing/confirmation and uncertain-result recovery. Check that tests prove the claimed behavior; local mocks do not establish deployed-program compatibility.
5. Run the narrowest meaningful tests/checks from `docs/commands.md`, expanding only for demonstrated risks. Report checks not run and why. Respect the distinction between normal tests and opt-in transactions/infrastructure.
6. Report findings with file/line evidence, impact and a concrete remedy: **BLOCKING** for correctness/security/acceptance failures, **non-blocking** for lesser issues, and **optional improvements** separately. If none are found, say so and state verification limits. Do not turn optional improvements into required scope or silently implement fixes during a review-only request.
