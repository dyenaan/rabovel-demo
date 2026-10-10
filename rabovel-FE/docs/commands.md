# Frontend commands

Run from `rabovel-FE/`. The declared package manager is `pnpm@10.33.2`.

```bash
pnpm install --frozen-lockfile
pnpm dev --port 3001
pnpm build
pnpm start --port 3001
```

`start` requires a successful production build. Builds use `next/font/google` and may need network access. Preserve existing lockfiles; this slice adds no frontend dependency.

## Environment

| Variable | Purpose |
| --- | --- |
| `RABOVEL_API_URL` | Server-only backend origin, locally `http://localhost:8080`. |
| `RABOVEL_PUBLIC_ORIGIN` | Exact frontend origin, locally `http://localhost:3001`. |
| `RABOVEL_GATEWAY_SECRET` | Server-only 64 hexadecimal characters, identical to backend setting. |
| `RABOVEL_STORAGE_ORIGIN` | Server-only HTTPS Supabase origin permitted by browser CSP for signed uploads. |
| `NEXT_PUBLIC_USE_MOCK_API` | `true` or `false`; defaults on in development, off in production. Production rejects `true`. |
| `NEXT_PUBLIC_WS_URL` | Optional WebSocket URL. |
| `NEXT_PUBLIC_ENVIRONMENT` | `development`, `staging` or `production`. |

Production gateway origins require HTTPS. The old `NEXT_PUBLIC_API_BASE_URL` setting is unused. Never expose the shared secret through public variables. See [rollout instructions](../../rabovel-BE/docs/plans/public-demo-hardening.md).

## Verification

```bash
pnpm test:gateway
pnpm test:security
./node_modules/.bin/tsc --noEmit --incremental false
pnpm lint -- src/lib/server/session-gateway.ts
pnpm build
```

Gateway tests use Node's built-in runner and TypeScript stripping (verified with Node 24); they exercise mock upstream responses, not a deployed backend. Replace the sample lint path with affected files. A clean checkout may need `./node_modules/.bin/next typegen` before checking generated route types. No automated browser suite covers the full devnet journey; smoke-test login, reload, logout and the issuer/investor flows after coordinated deployment.

## Responsive layout verification

Check at 320, 390, 768, and 1280 CSS pixels, including phone landscape:

- Marketing, login, onboarding, investor, issuer, and admin routes should fit the viewport. Wide financial tables scroll inside their own container.
- Use every bottom navigation item and the More drawer; issuer Overview and Verification links remain available on phones.
- On Trade, Place Order stays above the bottom navigation. Open its sheet, scroll the entire form, and check it with the software keyboard open.
- Open purchase/confirmation dialogs and long tab lists. Content and actions remain reachable with large text and short viewports.
- Check notched-device safe areas, form focus without automatic zoom, and both themes.

Type checks and production builds do not substitute for these browser checks.
