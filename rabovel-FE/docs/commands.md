# Frontend commands

Run from `rabovel-FE/`. Commands are grounded in `package.json` and local configuration; documentation setup does not establish a successful app build. The repository has a tracked `package-lock.json`, no `packageManager` field, and an existing untracked `pnpm-lock.yaml` at setup time. The commands below use npm; preserve other local lockfile work and do not silently migrate package managers.

## Install and environment

```bash
npm ci
```

This installs from the committed npm lockfile and replaces an existing dependency installation; use it when dependency setup is needed, not for documentation-only work. There is no repository Node engine/version pin.

`src/lib/env.ts` keeps local startup minimal. Mock mode needs no URL configuration;
real API mode requires only `NEXT_PUBLIC_API_BASE_URL`.

| Variable | Accepted value/purpose |
| --- | --- |
| `NEXT_PUBLIC_API_BASE_URL` | Required only when mocks are disabled; HTTP API base URL. |
| `NEXT_PUBLIC_WS_URL` | Optional WebSocket URL. When absent, realtime transport stays closed. |
| `NEXT_PUBLIC_ENVIRONMENT` | Optional; defaults to `development`. |
| `NEXT_PUBLIC_USE_MOCK_API` | `true` or `false`, defaults to `true`. |

These values are browser-visible; do not place secrets in them. Disabling mocks does not enable real auth or eliminate direct fixture imports; see [architecture.md](architecture.md).

## Dev/build/start

```bash
npm run dev
npm run build
npm run start
```

`start` requires a successful production build. The normal dev address is localhost:3000; when running the backend on that port, use `npm run dev -- --port 3001` and configure the frontend URL accordingly. Builds use `next/font/google` for Geist fonts and may require network access. Next generates route types and may maintain its marked agent-instruction block; preserve it.

## Verification

```bash
npm run lint -- src/features/orders/api/place-order.ts
npm run lint
./node_modules/.bin/tsc --noEmit --incremental false
```

Replace the sample lint path with the affected file(s); quote App Router paths containing parentheses. TypeScript configuration includes Next-generated types, and the app uses generated `LayoutProps`: a clean checkout may need `./node_modules/.bin/next typegen` before type checking. Consult the installed guide under `node_modules/next/dist/docs/01-app/03-api-reference/06-cli/next.md` for that CLI.

There is no test script, automated unit/browser test runner, formatter script, CI workflow, Rust build or validator setup in this repository. Do not invent `npm test` or treat lint as behavioral testing. For UI changes, exercise the affected route with the relevant role and mock/API mode, checking loading/error/empty/success states, narrow/mobile layout and keyboard behavior as applicable. Broaden to a build when routing, dependencies or server/client boundaries warrant it; report what actually ran.
