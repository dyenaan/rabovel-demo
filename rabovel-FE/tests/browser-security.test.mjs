import assert from "node:assert/strict";
import test from "node:test";
import { browserHeaders, contentSecurityPolicy } from "../src/lib/server/browser-security.ts";

const nonce = Buffer.from("unique-request-identifier").toString("base64");
test("production scripts require a nonce; external connections are explicitly scoped", () => {
  const policy = contentSecurityPolicy(nonce, { NODE_ENV: "production", RABOVEL_STORAGE_ORIGIN: "https://demo.supabase.co", NEXT_PUBLIC_WS_URL: "wss://stream.example.com/socket" });
  const script = policy.split(";").find((p) => p.trim().startsWith("script-src"));
  assert.ok(script.includes(`'nonce-${nonce}'`));
  assert.ok(script.includes("'strict-dynamic'"));
  assert.ok(!script.includes("unsafe-inline"));
  assert.ok(!script.includes("unsafe-eval"));
  assert.ok(policy.includes("connect-src 'self' https://demo.supabase.co wss://stream.example.com"));
  for (const directive of ["object-src 'none'", "base-uri 'none'", "frame-ancestors 'none'", "form-action 'self'", "upgrade-insecure-requests"]) assert.ok(policy.includes(directive));
  assert.notEqual(policy, contentSecurityPolicy(Buffer.from("another-request-identifier").toString("base64"), { NODE_ENV: "production" }));
});
test("invalid external origins and nonce injection fail closed", () => {
  for (const origin of ["http://storage.example.com", "https://user:pass@storage.example.com", "https://storage.example.com/path", "https://storage.example.com?nonce=foo"]) {
    assert.throws(() => contentSecurityPolicy(nonce, { NODE_ENV: "production", RABOVEL_STORAGE_ORIGIN: origin }));
  }
  assert.throws(() => contentSecurityPolicy("attacker'; script-src *", { NODE_ENV: "production" }));
  assert.throws(() => contentSecurityPolicy(nonce, { NODE_ENV: "production", NEXT_PUBLIC_WS_URL: "ws://stream.example.com" }));
  assert.ok(contentSecurityPolicy(nonce, { NODE_ENV: "development" }).includes("'unsafe-eval'"));
});
test("baseline headers prevent framing and sniffing while preserving wallet popups", () => {
  const headers = Object.fromEntries(browserHeaders(true).map(({key,value}) => [key,value]));
  assert.equal(headers["X-Frame-Options"], "DENY");
  assert.equal(headers["X-Content-Type-Options"], "nosniff");
  assert.equal(headers["Cross-Origin-Opener-Policy"], "same-origin-allow-popups");
  assert.equal(headers["Strict-Transport-Security"], "max-age=31536000");
  assert.ok(!browserHeaders(false).some(({key}) => key === "Strict-Transport-Security"));
});
