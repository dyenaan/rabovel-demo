# Quota & cap resets

The pipeline's AI steps run on subscription quotas, so the question is
never "how much did that cost" — it is "are we capped, and until when".
This file is the single source of truth. **Update it whenever a cap is hit
or a reset time changes** (one-line PR or direct edit by the repo owner).

## Current status

| Service | Used for | Plan | Resets | Notes |
| --- | --- | --- | --- | --- |
| ChatGPT / Codex | Builder | Plus (confirmed) | _fill in from Codex usage page_ | Plus has TWO meters: a rolling 5-hour window AND a weekly cap (hit either → wait). Schedule heavy builds around both; poll Codex `/status` per account, never hardcode resets. Pro ($100/$200) is exempt from the 5h window (weekly cap only). Dashboard/CLI is ground truth |
| Muse | Planner + reviewer | Free (weekly limit) | **Wed Oct 21, 2026, 3:38 PM MDT** (observed) | Ask Muse "check my usage" for a live read |

## How to check each cap

- **Codex:** ChatGPT → Codex → usage/limits page shows remaining quota and
  the reset time. Note both here when a run is planned.
- **Muse:** ask Muse in chat ("check my Muse usage") — it reads the
  live allowance and reset time. Observed pattern: free plan is a weekly
  limit resetting mid-afternoon MDT.

## Parking policy (when capped)

1. Label waiting tickets `quota-wait` and note which service + reset time
   in a comment. Do **not** silently switch a capped step to a paid API key.
2. When the reset passes, unlabel in ticket order (oldest first) and resume.
3. Muse (the planner/reviewer in chat) keeps a reminder for known reset
   times and will flag when caps are back — if a reset time moves, tell it
   so the reminder and this file stay in sync.

## API fallback (off by default)

Per-token API billing (e.g. ~$4 in / $20 out per million tokens for
frontier models) is the escape hatch for mid-sprint quota exhaustion only.
Enabling it = an edit to this file in the same PR that turns it on, with a
per-ticket spend cap written here first.
