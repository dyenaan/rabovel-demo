import assert from "node:assert/strict";
import test from "node:test";
import { hasAllowedRole, loginDestination, workspacesForUser } from "../src/features/auth/role-access.ts";

const origin = "https://demo.example.com";
const issuer = { id: "issuer", name: "Issuer", email: "issuer@example.com", role: "ISSUER", roles: ["ISSUER"] };
const investor = { ...issuer, role: "INVESTOR", roles: ["INVESTOR"] };

test("issuer login reaches the issuer workspace despite stale investor redirects", () => {
  for (const target of [null, "/dashboard", "/dashboard?old=1", "/assets", "/admin"]) {
    assert.equal(loginDestination(issuer, target, origin), "/issuer");
  }
  assert.equal(loginDestination(issuer, "/issuer/assets/ast_aa/edit?tab=details", origin), "/issuer/assets/ast_aa/edit?tab=details");
});

test("guards and navigation recognize all roles without granting missing roles", () => {
  const multiRole = { ...issuer, role: "ADMIN", roles: ["ADMIN", "ISSUER"] };
  assert.equal(hasAllowedRole(multiRole, ["ISSUER"]), true);
  assert.deepEqual(workspacesForUser(multiRole).map((item) => item.href), ["/issuer", "/admin"]);
  assert.equal(hasAllowedRole(investor, ["ISSUER"]), false);
  assert.equal(hasAllowedRole(null, ["ISSUER"]), false);
  assert.deepEqual(workspacesForUser(investor).map((item) => item.href), ["/dashboard"]);
  assert.deepEqual(workspacesForUser(issuer), [{ label: "Issuer workspace", href: "/issuer" }]);
  assert.deepEqual(workspacesForUser(null), []);
  assert.equal(hasAllowedRole({ ...issuer, roles: undefined }, ["ISSUER"]), true);
});

test("role-aware login keeps permitted deep links and rejects external or auth-loop redirects", () => {
  assert.equal(loginDestination(investor, "/trade?market=mkt_ngx_dangcem", origin), "/trade?market=mkt_ngx_dangcem");
  assert.equal(loginDestination(investor, "/issuer", origin), "/dashboard");
  for (const target of ["https://attacker.example.com", "//attacker.example.com", "/\\attacker.example.com", "/login", "/register"]) {
    assert.equal(loginDestination(issuer, target, origin), "/issuer");
  }
});
