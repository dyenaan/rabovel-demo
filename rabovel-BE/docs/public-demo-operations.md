# Public devnet demo operations

This runbook covers slice 4. Issuer quotas are intentionally excluded at the user's request. Slice 3's Turnstile and additional abuse/spend controls were deferred, not implemented. Existing general IP/session rate limits remain. This is a devnet demonstration with simulated verification, not a production brokerage.

## Deployment settings

Set Render `RABOVEL_DEPLOYMENT=public-devnet` before deploying this slice. Hosted origins otherwise fail startup. The profile requires an HTTPS frontend origin, gateway secret, Postgres, Redis, Supabase, all existing Solana signer/derivation settings, cNGN mint, `RABOVEL_DEMO_MODE=true` and `RABOVEL_SOLANA_NETWORK=devnet`. Kafka must be unset. It validates configuration and checks the RPC's devnet genesis hash before running database migrations. An unavailable or non-devnet RPC prevents startup. The genesis check trusts the configured RPC provider and is performed at startup, not continuously.

Debug and release builds are both supported. For native Render hosting, you can start with `cargo run -p rabovel-api`, or build with `cargo build -p rabovel-api --release --locked` and start `./target/release/rabovel-api`. Set the health-check path to `/health`. The API uses Render's `PORT` unless `RABOVEL_BIND_ADDR` explicitly overrides it; local default remains 8080. See [Render health checks](https://render.com/docs/health-checks).

In Vercel retain the slice-1 server environment (`RABOVEL_API_URL`, `RABOVEL_PUBLIC_ORIGIN`, `RABOVEL_GATEWAY_SECRET`) and `NEXT_PUBLIC_USE_MOCK_API=false`. Add server-only `RABOVEL_STORAGE_ORIGIN=https://<your-project>.supabase.co`, matching the backend Supabase origin, to permit direct signed image uploads. No storage secret belongs in frontend configuration. If absent, external upload connections are blocked by CSP. An optional `NEXT_PUBLIC_WS_URL` must use WSS in production; omit it when no WebSocket server is deployed.

Nonce-based CSP requires dynamic page rendering. HTML is private/no-store, and next-themes receives the nonce. Production scripts do not permit unsafe-inline or unsafe-eval; existing chart/motion/Radix inline styles remain allowed. Images allow HTTPS, data and blob sources to support issuer images and local previews. Wallet popups remain permitted. This tradeoff reduces static/CDN caching and must be checked in a real browser, especially Phantom signing.

## Secrets and networking review

- Use the internal Postgres and Key Value URLs for Render services in the same region. Keep their external inbound allowlists empty if no external administrative client needs access. Private-network connections are unaffected by external inbound rules. Render Key Value internal connections need not include a password; do not invent credentials. See [private networking](https://render.com/docs/private-network), [inbound rules](https://render.com/docs/inbound-ip-rules) and [Key Value](https://render.com/docs/key-value).
- Vercel-to-Render API traffic remains HTTPS with the shared gateway secret. Do not restrict the API to your laptop's IP; visitors arrive through the frontend server. Do not assume CORS restricts non-browser clients.
- Keep KV `noeviction` and the existing small memory allowance. Redis/Postgres operation errors now return generic 503 responses. No security-state fallback is enabled after a failure. A full KV store may block even reads that require a rate-limit write; do not switch to eviction to hide the problem.
- Use devnet-only, minimally funded signer wallets, protect the derivation secret and avoid exposing hosted secrets to untrusted previews. Review all `NEXT_PUBLIC_*` values; none may contain a secret. Hosted validation checks URL form, not actual firewall settings or provider account permissions.
- Review the migration/role changes from slice 2 and obtain a restorable database backup before deployment. Do not drop tables or clear Docker/database volumes as part of this rollout.

## Verification

Run the automated suites and a successful production frontend build before release. From `rabovel-FE`, run the public smoke script against canonical HTTPS origins:

```bash
pnpm test:security
pnpm test:gateway
pnpm smoke:public https://<frontend-host> https://<render-api-host>
```

The script performs public reads and deliberately rejected mutations only. It checks liveness, direct API rejection, gateway/Redis catalog access, signed-out cookies, origin checks, route restrictions, browser headers and fresh matching script nonces. It does not create accounts or submit transactions. It prints no response bodies, cookies or secrets.

Then use a browser to test both existing users: login, reload, logout, session expiry, theme switching, issuer image preview/upload, draft editing, and Phantom wallet linking/signing. Check the console for CSP violations. Any actual devnet transaction is a separate manual smoke action; inspect its signature before retrying an ambiguous result. Test mobile or wallet popup flows as applicable.

## Monitoring

API requests emit one JSON line with event, request ID, HTTP method, route template, status and duration. Query strings, raw asset/user IDs, IPs, bodies, authorization, cookies, gateway secrets and provider error details are excluded. Health polling is omitted. Gateway request IDs are UUID-validated; invalid ones are replaced. Internal failure details are not returned to clients.

Configure external monitoring for `/health` and the frontend `/api/instruments` path, and alerts for service restarts, sustained 5xx responses, elevated latency and KV memory/rejected writes. These checks have different meanings: `/health` is process liveness; the catalog exercises the gateway and Redis limiter, but neither is a continuous Postgres or Solana readiness check. Monitor provider status and database connectivity separately. Keep synthetic catalog polling comfortably below the existing per-IP limit. No monitoring vendor or alert destination was configured by this code change.

## Recovery

1. For a failed rollout, inspect the deployment error and request IDs. Missing profile settings or RPC identity failures must be corrected; do not disable the hosted profile to force startup.
2. For KV exhaustion/outage, restore availability or capacity and inspect TTL/memory behavior. Do not flush all keys or enable eviction as a routine fix. Expired sessions can require users to log in again; issued wallet challenges must never be restored from a stale snapshot.
3. For database failure, pause user actions and restore connectivity. Restore a verified backup into an isolated database first and verify schema/migration state before switching the service. Redis sessions are separate from Postgres backups.
4. For a leaked gateway secret, rotate it on both Vercel and Render in a coordinated maintenance window and verify direct-API rejection. For credential/session compromise, revoke the affected sessions through an operator-controlled process. No public bulk-revocation endpoint is introduced.
5. For signer compromise, pause chain mutation traffic and rotate the relevant keys. Issuer/admin authorities and deterministic mint derivation can bind existing assets to old keys; reconcile those assets before changing signer or derivation configuration. Never automatically retry a potentially submitted transaction.
6. Roll back frontend/backend versions together when contracts changed. Do not restore the removed email-based role grants or roll back to a binary that grants them. Migrations are not automatically reversed by a binary rollback.
