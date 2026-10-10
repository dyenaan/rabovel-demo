// Read-only public deployment checks. Never logs cookies, secrets or bodies.
// Usage: node scripts/smoke-public-demo.mjs https://frontend.example https://api.example
import assert from "node:assert/strict";

function origin(value) {
  const url = new URL(value);
  assert.equal(url.protocol, "https:", "Use the canonical HTTPS origin");
  assert.ok(!url.username && !url.password && url.pathname === "/" && !url.search && !url.hash, "Use origins without credentials, paths or queries");
  return url.origin;
}
async function main() {
  const frontend = origin(process.argv[2]);
  const backend = origin(process.argv[3]);
  const request = (url, init) => fetch(url, { ...init, redirect: "manual", cache: "no-store", signal: AbortSignal.timeout(15_000) });
  const health = await request(`${backend}/health`);
  assert.equal(health.status, 200, "Backend health");
  assert.equal((await health.json()).status, "ok", "Backend health payload");
  for (const path of ["/auth/me", "/instruments"]) {
    const response = await request(`${backend}${path}`, { headers: { "x-forwarded-for": "203.0.113.1", "x-rabovel-client-ip": "203.0.113.1" } });
    assert.equal(response.status, 403, `Backend gateway boundary: ${path}`);
    assert.equal(response.headers.get("cache-control"), "no-store");
  }
  const catalog = await request(`${frontend}/api/instruments`);
  assert.equal(catalog.status, 200, "Gateway-to-backend catalog and Redis rate limiter");
  const me = await request(`${frontend}/api/auth/me`);
  assert.equal(me.status, 401, "Signed-out frontend session");
  assert.equal(me.headers.get("cache-control"), "no-store");
  const clear = me.headers.get("set-cookie") ?? "";
  for (const value of ["__Host-rabovel-session=", "HttpOnly", "Secure", "SameSite=Lax", "Max-Age=0"]) assert.ok(clear.includes(value), "Safe session clearing cookie");
  const csrf = await request(`${frontend}/api/auth/login`, { method: "POST", headers: { origin: "https://invalid.example", "content-type": "application/json" }, body: "{}" });
  assert.equal(csrf.status, 403, "Cross-origin mutation rejection");
  const webhook = await request(`${frontend}/api/kyc/webhook`, { method: "POST", headers: { origin: frontend, "content-type": "application/json" }, body: "{}" });
  assert.equal(webhook.status, 404, "Gateway route allowlist");
  let previous;
  for (let attempt = 0; attempt < 2; attempt++) {
    const page = await request(`${frontend}/login`);
    assert.equal(page.status, 200, "Login page");
    assert.equal(page.headers.get("x-frame-options"), "DENY");
    assert.equal(page.headers.get("x-content-type-options"), "nosniff");
    assert.ok(page.headers.get("strict-transport-security"));
    assert.equal(page.headers.get("x-powered-by"), null);
    assert.ok(page.headers.get("cache-control")?.includes("no-store"));
    const policy = page.headers.get("content-security-policy") ?? "";
    const nonce = policy.match(/'nonce-([^']+)'/)?.[1];
    assert.ok(nonce, "CSP nonce");
    assert.notEqual(nonce, previous, "Fresh nonce for each page response");
    const scriptPolicy = policy.split(";").find((part) => part.trim().startsWith("script-src"));
    assert.ok(scriptPolicy && !scriptPolicy.includes("unsafe-inline") && !scriptPolicy.includes("unsafe-eval"), "Strict production script policy");
    const html = await page.text();
    const scripts = [...html.matchAll(/<script\b[^>]*>/gi)].map(([tag]) => tag).filter((tag) => !/type="application\/(json|ld\+json)"/.test(tag));
    assert.ok(scripts.length > 0, "Rendered scripts");
    assert.ok(scripts.every((tag) => tag.includes(`nonce="${nonce}"`)), "Rendered scripts match the policy nonce");
    previous = nonce;
  }
  process.stdout.write("PASS: health, gateway boundary, session cookie, CSRF, route restrictions, headers and fresh CSP nonces.\n");
  process.stdout.write("Still required: browser login/reload/logout, image upload, theme switching and Phantom signing.\n");
}
main().catch((error) => { process.stderr.write(`FAIL: ${error instanceof Error ? error.message : "deployment check failed"}\n`); process.exitCode = 1; });
