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

Route-group parentheses do not appear in URLs. The root layout loads Geist fonts/global CSS and `providers.tsx`, which supplies theme, React Query, auth-token wiring, tooltips and toasts. React Query owns fetched data/cache; Zustand owns session, onboarding, sidebar and user preferences. Charts use Recharts; forms commonly use React Hook Form and Zod.

## Data and integration boundaries

Typical flow: route → feature component → query/mutation hook → feature API → fixture or `apiClient` → shared display. Query keys live in `src/lib/query-keys.ts`; mutations invalidate affected queries. Defaults are 30-second query freshness, one query retry and no mutation retry, with feature overrides.

`src/lib/env.ts` validates public environment variables at import time. `NEXT_PUBLIC_USE_MOCK_API` defaults to true and selects fixture branches and a simulated WebSocket transport. Real HTTP calls use a configured API base URL, bearer token callback, request IDs and optional idempotency headers; they expect `{ data, meta? }` success envelopes. TypeScript casts do not validate server payloads at runtime.

`src/lib/websocket/` shares one transport and manages channel listeners/reconnection. Hooks currently expose market, order-book and public-trade streams. Order/settlement channel names exist, but the mock transport only emits those three market streams; settlement views use queries, not a settlement subscription.

The flag is not a complete integration switch: login/registration call mock auth directly; wallet connection mutates fixtures; onboarding submission is simulated; several admin/compliance/reserve views import mocks directly. There are no application API route handlers or backend services in this folder. `next.config.ts` contains no API proxy configuration.

## Authentication and backend compatibility

`auth-store.ts` persists user/token in browser storage; `AuthProvider` registers a token getter with the HTTP client. `RequireAuth` waits for hydration and redirects signed-out users; `RoleGuard` controls visible content. These are presentation controls, not a server authorization boundary. Onboarding has its own layout without those guards.

Current frontend contracts are not confirmed against the sibling backend. Concrete differences include mock email/password login versus backend Google OIDC, frontend `/portfolio/summary` versus backend `/portfolio`, camelCase/uppercase order DTOs versus backend snake_case/lowercase values, and frontend wrapped responses versus backend direct DTO responses. Resolve each relevant contract during integration; changing an environment URL alone is insufficient. Production deployment, session lifecycle and WebSocket server compatibility remain unresolved.
