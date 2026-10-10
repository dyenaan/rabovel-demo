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
| `NEXT_PUBLIC_USE_MOCK_API` | `true` or `false`; defaults on in development, off in production. Production rejects `true`. |
| `NEXT_PUBLIC_WS_URL` | Optional WebSocket URL. |
| `NEXT_PUBLIC_ENVIRONMENT` | `development`, `staging` or `production`. |

Production gateway origins require HTTPS. The old `NEXT_PUBLIC_API_BASE_URL` setting is unused. Never expose the shared secret through public variables. See [rollout instructions](../../rabovel-BE/docs/plans/public-demo-hardening.md).

## Verification

```bash
pnpm test:gateway
./node_modules/.bin/tsc --noEmit --incremental false
pnpm lint -- src/lib/server/session-gateway.ts
pnpm build
```

Gateway tests use Node's built-in runner and TypeScript stripping (verified with Node 24); they exercise mock upstream responses, not a deployed backend. Replace the sample lint path with affected files. A clean checkout may need `./node_modules/.bin/next typegen` before checking generated route types. No automated browser suite covers the full devnet journey; smoke-test login, reload, logout and the issuer/investor flows after coordinated deployment.
