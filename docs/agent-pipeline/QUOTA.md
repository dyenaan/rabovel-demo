# Quota & cap resets

The pipeline's AI steps run on subscription quotas, so the question is
never "how much did that cost" — it is "are we capped, and until when".
This file is the single source of truth. **Update it whenever a cap is hit
or a reset time changes** (one-line PR or direct edit by the repo owner).

## Current status

| Service | Used for | Plan | Resets | Notes |
| --- | --- | --- | --- | --- |
| ChatGPT / Codex | Builder | Plus (confirmed) | _fill in from Codex usage page_ | Plus tier: weekly rolling caps; no Pro-style 5h gate logic needed. Dashboard is ground truth |
| Muse | Planner + reviewer option | Free (weekly limit) | **Wed Oct 21, 2026, 3:38 PM MDT** (observed) | Ask Muse "check my usage" for a live read |
| OpenCode | Builder / reviewer | rides provider login above | follows that provider | Headless CI auth is the fiddly part; prefer Codex cloud for builds |

## How to check each cap

- **Codex:** ChatGPT → Codex → usage/limits page shows remaining quota and
  the reset time. Note both here when a run is planned.
- **Muse:** ask Muse in chat ("check my Muse usage") — it reads the
  live allowance and reset time. Observed pattern: free plan is a weekly
  limit resetting mid-afternoon MDT.
- **OpenCode:** quota is whatever provider it is logged into; check that
  provider.

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
