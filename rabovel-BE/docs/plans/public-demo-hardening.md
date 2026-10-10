# Public devnet demo hardening

## Slice 1: cookie sessions and authenticated gateway

PR scope: replace browser-persisted bearer tokens with a Next.js Node gateway and HttpOnly cookies; require a shared gateway secret at the Render API boundary. Existing user-session and role checks remain. This is one hardening slice, not a completed public-demo security review.

The gateway uses a fixed backend origin and route/method allowlist, exact mutation Origin checks, JSON-only mutations, bounded request/response bodies, no response caching or redirect following, fresh trusted headers, and generic upstream failure messages. Logout clears the browser cookie even when upstream revocation fails; an unrevoked backend token still expires at its existing 15-minute deadline. Vercel client-IP metadata is forwarded only by the authenticated gateway. Production assumes Vercel hosting; consult [Vercel's request-header contract](https://vercel.com/docs/headers/request-headers). Additional proxies require a reviewed trust configuration.

Local `.env` and `.env.local` use the same randomly generated secret, API port 8080 and frontend origin 3001. No production setting or deployment was changed. No data volumes were changed by this slice.

## Coordinated rollout

1. Generate a fresh hosted secret using `openssl rand -hex 32` and store it as `RABOVEL_GATEWAY_SECRET` in both Render and Vercel server environments. Never expose it in a `NEXT_PUBLIC_*` variable, git, logs or a PR.
2. In Vercel set `RABOVEL_API_URL=https://<render-api-host>`, `RABOVEL_PUBLIC_ORIGIN=https://<demo-frontend-host>` and `NEXT_PUBLIC_USE_MOCK_API=false`. Set the same exact public origin in Render. Use the existing Render release build and private Postgres/KV URLs.
3. Coordinate both deployments in a maintenance window: the old frontend calls the API directly and becomes incompatible once the backend gate is enabled. Missing/malformed secrets fail closed. No Render IP allowlist for Vercel is assumed; the application gate authenticates this boundary.
4. Existing localStorage credentials are removed on load. Users must sign in again. Confirm browser login responses contain no bearer token and cookies have HttpOnly, Secure, SameSite=Lax, Path=/ and no Domain.
5. Smoke-test login, reload/session restoration, logout, expired-session handling, issuer asset editing and Phantom wallet/purchase flows. Confirm direct Render `/auth/me` is 403 without gateway headers and `/health` is 200. Confirm forged forwarded headers cannot bypass rate limits.
6. Keep production secrets out of untrusted preview builds. Use an isolated preview backend and separate secret, or leave previews without live API access.

Mutations have no automatic gateway retries. Long chain operations have a 280-second upstream timeout and a 300-second route duration setting; the actual hosting plan's function limit still applies. On timeout, check operation status before retrying. `/kyc/webhook`, Google OAuth callbacks and debug documentation are deliberately absent from the browser gateway allowlist; integrations needing these require a separately scoped design.

## Verification and limits

Automated checks cover gateway cookies, Origin validation, credentials/header stripping, route/method restrictions, bounds, logout and failure handling; backend tests cover secret enforcement, IP trust and existing auth/application behavior. These checks do not substitute for the coordinated live smoke test above. No live Render/Vercel deployment, Redis integration restart test or on-chain transaction was performed for this slice.

Checks completed for the slice:

- `cargo test -p rabovel-api --lib --locked --offline`: 26 passed.
- `cargo clippy -p rabovel-api --all-targets --locked --offline -- -D warnings` and `cargo fmt -p rabovel-api --check`: passed.
- `node --experimental-strip-types tests/session-gateway.test.mjs`: 13 passed.
- Next route type generation and TypeScript checking: passed.
- ESLint on changed frontend source: no errors; one existing React Hook Form compiler warning in registration.
- Production build: blocked downloading existing Geist Google Fonts in the restricted network environment; the requested network-enabled retry was declined. A successful production build remains a deployment check.

## Slice 2: public signup and role authorization

Password signup requires an explicit `account_type` (`investor` or `issuer`) and creates exactly one public role. Unknown fields, multiple roles and administrative/operational role selections are rejected. Password login no longer assigns roles from email; the legacy demo email settings are ignored. The UI provides the same two choices. Issuers must complete the existing simulated organization onboarding before asset creation; this does not establish verified legal identity.

Migration `0006_restrict_password_roles.sql` normalizes existing password accounts: accounts containing an issuer role keep only issuer, all others become investor/trader; permissions are reduced to the corresponding public set. This intentionally removes previous password admin/compliance/operations grants. Users, credentials, organizations and assets are preserved. OIDC-only users are unchanged. Existing sessions reference current database roles, so the change applies without waiting for expiry. Do not restore old role grants when rolling back the binary. Review affected account IDs and role arrays before deployment; do not copy password hashes into review output.

Deploy backend and frontend together: old registration clients omit the newly required account type and will receive 422. Existing password login continues working. Remove obsolete `RABOVEL_DEMO_ADMIN_EMAIL` and `RABOVEL_DEMO_ISSUER_EMAIL` environment variables. Existing investors cannot upgrade in place through signup; use a separate issuer account until an explicit account-conversion flow is designed. No public role-change or admin-provisioning endpoint is introduced.

Authorization regression tests cover public role selection and persistence, attempted privilege injection, pending issuer restrictions, and access to another issuer's asset across read/edit/delete/setup/inventory/backing/listing/image endpoints. Image confirmation now checks ownership and draft state before calling storage. The migration has a separate opt-in test using temporary Postgres tables only.

Slice 2 verification:

- API unit/regression suite: 29 passed, including the three new signup/authorization tests. Strict Clippy for API and infrastructure (all targets), formatting and diff whitespace checks passed.
- Frontend TypeScript checking: passed. Changed-file ESLint: no errors, one existing React Hook Form compiler warning.
- The Postgres migration test compiled, but its connection was blocked by sandbox permissions; the requested local-database retry was declined. Database execution remains unverified and must pass before deployment. No migration was applied to existing local or hosted data.
- Browser signup/onboarding and hosted smoke tests were not performed. Production build remains a deployment check from slice 1.

To verify the migration against an available local Postgres instance, supply its URL through `RABOVEL_TEST_DATABASE_URL` and run from `rabovel-BE`:

```bash
cargo test -p infrastructure --test password_role_migration --locked -- --ignored
```

This test uses temporary tables in a rolled-back transaction; it does not migrate the configured database.

Account/IP signup abuse controls, spend limits and quotas remain slice 3. Do not treat this slice as approval to advertise unrestricted signup before those controls are deployed.

## Remaining slices

- Slice 3: Turnstile, account/IP abuse controls, small issuer quotas and devnet transaction/spend limits, including failure behavior when the 25 MB KV store is full.
- Slice 4: explicit hosted demo configuration, CSP/security headers, secrets and private-network checks, monitoring, recovery and operational smoke tests.

Finish and review each slice in its own PR before beginning the next.
