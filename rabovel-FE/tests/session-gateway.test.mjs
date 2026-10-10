import assert from "node:assert/strict";
import test from "node:test";
import { gatewayConfig, handleGateway } from "../src/lib/server/session-gateway.ts";

const config = {
  backendOrigin: "https://backend.example.com",
  publicOrigin: "https://demo.example.com",
  secret: "a".repeat(64),
  production: true,
};
const token = "b".repeat(64);
const makeRequest = (path, method = "GET", headers = {}, body) => new Request(`${config.publicOrigin}/api${path}`, {
  method,
  headers: {
    origin: config.publicOrigin,
    "content-type": "application/json",
    "x-vercel-forwarded-for": "203.0.113.7",
    cookie: `__Host-rabovel-session=${token}`,
    ...headers,
  },
  body,
});

test("production config requires HTTPS, a fixed origin, and a strong secret", () => {
  const environment = { NODE_ENV: "production", RABOVEL_API_URL: config.backendOrigin,
    RABOVEL_PUBLIC_ORIGIN: config.publicOrigin, RABOVEL_GATEWAY_SECRET: config.secret };
  assert.deepEqual(gatewayConfig(environment), config);
  for (const patch of [
    { RABOVEL_API_URL: "http://backend.example.com" },
    { RABOVEL_API_URL: "https://backend.example.com/private" },
    { RABOVEL_API_URL: "https://user:password@backend.example.com" },
    { RABOVEL_GATEWAY_SECRET: "weak" },
    { RABOVEL_PUBLIC_ORIGIN: undefined },
  ]) assert.throws(() => gatewayConfig({ ...environment, ...patch }));
});

test("login sets a host-only HttpOnly cookie and removes the bearer from JSON", async () => {
  const response = await handleGateway(makeRequest("/auth/login", "POST", {}, "{}"), config,
    async () => Response.json({ access_token: token, token_type: "Bearer", expires_in: 900 }));
  assert.deepEqual(await response.json(), { expires_in: 900 });
  const cookie = response.headers.get("set-cookie");
  for (const part of ["__Host-rabovel-session=", "HttpOnly", "Secure", "SameSite=Lax", "Path=/", "Max-Age=900"]) assert.ok(cookie.includes(part));
  assert.ok(!cookie.includes("Domain="));
  assert.equal(response.headers.get("cache-control"), "no-store");
});

test("registration also strips credentials and rejects malformed session responses", async () => {
  for (const body of [{ access_token: "bad", expires_in: 900 }, { access_token: token, expires_in: 9999 }]) {
    const response = await handleGateway(makeRequest("/auth/register", "POST", {}, "{}"), config,
      async () => Response.json(body));
    assert.equal(response.status, 503);
    assert.equal(response.headers.get("set-cookie"), null);
    assert.ok(!(await response.text()).includes(token));
  }
});

test("cookie restores a session; caller auth, IP, secret, host, and cookies are not forwarded", async () => {
  const response = await handleGateway(makeRequest("/auth/me", "GET", {
    authorization: "Bearer attacker", "x-rabovel-gateway-secret": "attacker",
    "x-rabovel-client-ip": "198.51.100.1", "x-forwarded-for": "198.51.100.1",
    host: "attacker.example.com",
  }), config, async (url, options) => {
    assert.equal(url, `${config.backendOrigin}/auth/me`);
    assert.equal(options.headers.get("authorization"), `Bearer ${token}`);
    assert.equal(options.headers.get("x-rabovel-gateway-secret"), config.secret);
    assert.equal(options.headers.get("x-rabovel-client-ip"), "203.0.113.7");
    for (const name of ["host", "cookie", "x-forwarded-for"]) assert.equal(options.headers.get(name), null);
    assert.equal(options.redirect, "manual");
    assert.equal(options.cache, "no-store");
    return Response.json({ user_id: "usr_1" }, { headers: { "set-cookie": "untrusted=true" } });
  });
  assert.deepEqual(await response.json(), { user_id: "usr_1" });
  assert.equal(response.headers.get("set-cookie"), null);
});

test("mutations require the exact configured Origin and JSON before contacting backend", async () => {
  for (const origin of ["https://attacker.example.com", "null", "", "https://demo.example.com.attacker.com"]) {
    const response = await handleGateway(makeRequest("/auth/login", "POST", { origin }, "{}"), config,
      async () => assert.fail("backend must not be contacted"));
    assert.equal(response.status, 403);
  }
  const response = await handleGateway(makeRequest("/auth/login", "POST", { "content-type": "text/plain" }, "{}"), config,
    async () => assert.fail("backend must not be contacted"));
  assert.equal(response.status, 415);
});

test("unknown routes, encoded paths, webhooks, debug routes, and unsupported methods are blocked", async () => {
  for (const path of ["/kyc/webhook", "/kyc/_sandbox/advance", "/auth/google/start", "/swagger-ui", "/issuer/assets/ast_aa%2fsetup", "//attacker.example.com"]) {
    const response = await handleGateway(makeRequest(path), config, async () => assert.fail("no proxy request"));
    assert.equal(response.status, 404);
  }
  const response = await handleGateway(makeRequest("/auth/me", "POST", {}, "{}"), config,
    async () => assert.fail("no proxy request"));
  assert.equal(response.status, 405);
});

test("missing cookies and invalid trusted IP metadata fail closed", async () => {
  const response = await handleGateway(makeRequest("/auth/me", "GET", { cookie: "" }), config,
    async () => assert.fail("no proxy request"));
  assert.equal(response.status, 401);
  for (const ip of ["", "invalid", "203.0.113.1, 198.51.100.1"]) {
    const response = await handleGateway(makeRequest("/auth/me", "GET", { "x-vercel-forwarded-for": ip }), config,
      async () => assert.fail("no proxy request"));
    assert.equal(response.status, 400);
  }
});

test("expired backend session clears the browser cookie", async () => {
  const response = await handleGateway(makeRequest("/auth/me"), config,
    async () => Response.json({ code: "unauthorized" }, { status: 401 }));
  assert.equal(response.status, 401);
  assert.ok(response.headers.get("set-cookie").includes("Max-Age=0"));
});

test("logout forwards revocation and clears the cookie even if backend is unavailable", async () => {
  for (const unavailable of [false, true]) {
    const response = await handleGateway(makeRequest("/auth/logout", "POST"), config, async (url, options) => {
      assert.equal(url, `${config.backendOrigin}/auth/logout`);
      assert.equal(options.headers.get("authorization"), `Bearer ${token}`);
      if (unavailable) throw new Error("private connection details");
      return new Response(null, { status: 204 });
    });
    assert.equal(response.status, unavailable ? 503 : 204);
    assert.ok(response.headers.get("set-cookie").includes("Max-Age=0"));
    assert.ok(!(await response.text()).includes("private connection details"));
  }
});

test("oversized request is bounded even without a Content-Length header", async () => {
  const response = await handleGateway(makeRequest("/issuer/assets", "POST", {}, "x".repeat(16 * 1024 + 1)), config,
    async () => assert.fail("no proxy request"));
  assert.equal(response.status, 413);
});

test("redirects and backend internal failures are not exposed", async () => {
  for (const status of [302, 500]) {
    const response = await handleGateway(makeRequest("/auth/me"), config,
      async () => new Response("private error", { status, headers: { location: "https://attacker.example.com" } }));
    assert.equal(response.status, status === 302 ? 502 : 503);
    assert.equal(response.headers.get("location"), null);
    assert.ok(!(await response.text()).includes("private error"));
  }
});

test("local development uses a non-Secure localhost cookie and a fixed local client IP", async () => {
  const local = { ...config, production: false };
  const response = await handleGateway(makeRequest("/auth/login", "POST", { "x-vercel-forwarded-for": "attacker" }, "{}"), local,
    async (_, options) => {
      assert.equal(options.headers.get("x-rabovel-client-ip"), "127.0.0.1");
      return Response.json({ access_token: token, expires_in: 900 });
    });
  const cookie = response.headers.get("set-cookie");
  assert.ok(cookie.startsWith("rabovel-session="));
  assert.ok(!cookie.includes("Secure"));
});

test("query parameters preserve the fixed backend origin and have a bounded size", async () => {
  const query = "?next=https%3A%2F%2Fattacker.example.com";
  const response = await handleGateway(makeRequest(`/instruments${query}`), config, async (url) => {
    assert.equal(url, `${config.backendOrigin}/instruments${query}`);
    return Response.json([]);
  });
  assert.equal(response.status, 200);
  const oversized = await handleGateway(makeRequest(`/instruments?q=${"a".repeat(2048)}`), config,
    async () => assert.fail("backend must not be contacted"));
  assert.equal(oversized.status, 404);
});
