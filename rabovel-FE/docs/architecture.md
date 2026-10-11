# Current frontend architecture

One Next.js application, not a package workspace. `package.json` pins Next 16.3.3 and React 19.2.8; TypeScript is strict, Tailwind uses the v4 PostCSS plugin. Read the relevant installed Next guide before framework code changes, as required by `AGENTS.md`.

| Area | Responsibility |
| --- | --- |
| `src/app/(marketing)` | Public landing/about/how-it-works routes. |
| `src/app/(auth)`, `(onboarding)` | Login/register/verification/reset screens and local onboarding wizard. |
| `src/app/(dashboard)`, `(primary-market)` | Investor routes wrapped by `RequireAuth`; includes trading, orders, holdings, settlements, subscriptions/redemptions. |
| `src/app/admin` | Operations routes wrapped by `RoleGuard` for ADMIN, COMPLIANCE and OPERATIONS. |
| `src/features/<feature>` | Feature UI, query/mutation hooks, API adapters and form schemas where present. |
| `src/components` | Radix-based `ui/` primitives, reusable `shared/` views, `layout/` navigation and `financial/` displays. |
| `src/lib` | Environment validation, HTTP client, query keys, formatters and WebSocket transports. |
| `src/types`, `src/mocks`, `src/stores` | Frontend contracts, fixture data, and Zustand browser state respectively. |

Route-group parentheses do not appear in URLs. The root layout loads Geist fonts/global CSS and `providers.tsx`, which supplies theme, React Query, session restoration, tooltips and toasts. React Query owns fetched data/cache; Zustand owns session, onboarding, sidebar and user preferences. Charts use Recharts; forms commonly use React Hook Form and Zod.

## Data and integration boundaries

Typical flow: route → feature component → query/mutation hook → feature API → fixture or `apiClient` → shared display. Query keys live in `src/lib/query-keys.ts`; mutations invalidate affected queries. Defaults are 30-second query freshness, one query retry and no mutation retry, with feature overrides.

`src/lib/env.ts` validates browser-visible configuration. Mock mode defaults on in development and off in production; production rejects mock authentication. Real adapters call the same-origin `/api/*` gateway. Server-only configuration fixes the backend origin and shared gateway secret. Legacy adapters still expect wrapped responses; primary demo adapters use backend DTOs. TypeScript casts do not validate response payloads.

`src/lib/websocket/` shares one transport and manages channel listeners/reconnection. Hooks currently expose market, order-book and public-trade streams. Order/settlement channel names exist, but the mock transport only emits those three market streams; settlement views use queries, not a settlement subscription.

The flag is not a complete integration switch: several admin/compliance/reserve views still import fixtures. The Node route handler at `src/app/api/[...path]/route.ts` forwards only allowlisted routes and methods to a fixed backend. It bounds bodies and responses, rejects redirects, requires exact configured Origin plus JSON on mutations, and rebuilds upstream headers. In production it trusts Vercel's client-IP metadata and fails closed when absent; deployment outside Vercel needs a separately reviewed IP trust configuration.

## Authentication and backend compatibility

The gateway stores backend bearer credentials in a host-only HttpOnly, SameSite=Lax cookie (`__Host-rabovel-session`, Secure in production). Login/register JSON omits the credential. `auth-store.ts` holds only the user in memory; `AuthProvider` restores it through `/api/auth/me` and removes legacy persisted credentials. Logout revokes the backend session and clears the cookie; backend 401 responses also clear it. Sessions retain the existing 15-minute lifetime without automatic refresh.

`RequireAuth` and `RoleGuard` are presentation controls. Backend authorization remains necessary on every protected route. Gateway authentication proves the request came through the frontend server; it does not grant a user role. Query caches are cleared on login/logout. Existing sessions persisted only in localStorage require a fresh login after deployment.

See the [hardening rollout](../../rabovel-BE/docs/plans/public-demo-hardening.md). Role assignment, public issuer safeguards, spend quotas and browser security policy remain later slices. WebSocket transport and fixture-backed screens are not enabled by this gateway.

## Investor account and purchase presentation

The `/wallet` route is labeled Account & Payments (Payments in mobile navigation). Investor setup, the asset catalog and purchase review emphasize available cNGN balance, units and holdings. Wallet addresses, network and mint/token-account identifiers sit in expandable details; the demo funding address stays visible because the faucet still requires it. Demo and simulated-price disclosures remain visible. Phantom still verifies wallet ownership and signs purchases; the confirmation screen explains that approval and potential network fees. Embedded signing, fee sponsorship and bank/card funding are not implemented by this presentation change.

## Browser security boundary

`src/proxy.ts` generates a fresh script nonce and CSP for page requests. The root layout reads the nonce, making pages dynamic, and passes it through to next-themes. Nonce-bearing HTML is private/no-store. Baseline headers are configured in `next.config.ts`. External browser connections are limited to the configured storage origin and optional WebSocket origin; issuer images remain HTTPS-capable. Inline styles remain permitted for existing UI libraries, while production inline scripts require a nonce. See the [operations runbook](../../rabovel-BE/docs/public-demo-operations.md) for deployment settings and browser verification.
