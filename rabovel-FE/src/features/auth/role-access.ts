import type { UserRole } from "../../types/common.types";
import type { AuthUser } from "./types/auth.types";

/** Check every server-provided role; primary role only selects the default home. */
export function hasAllowedRole(user: AuthUser | null, allow: readonly UserRole[]): boolean {
  if (!user) return false;
  return [user.role, ...(user.roles ?? [])].some((role) => allow.includes(role));
}

export const homeForRole = (role: UserRole) =>
  ["ADMIN", "COMPLIANCE", "OPERATIONS"].includes(role) ? "/admin" : role === "ISSUER" ? "/issuer" : "/dashboard";

const investorPaths = new Set([
  "dashboard", "assets", "markets", "orders", "watchlist", "wallet", "trade", "trades",
  "security", "documents", "settlements", "primary-market", "redeem", "subscribe",
]);

/** Preserve same-origin deep links only when the account can enter their area. */
export function loginDestination(user: AuthUser, requested: string | null, origin: string): string {
  const home = homeForRole(user.role);
  if (!requested) return home;
  try {
    const url = new URL(requested, origin);
    if (url.origin !== origin) return home;
    if (url.pathname === "/dashboard" && user.role !== "INVESTOR") return home;
    const area = url.pathname.split("/")[1];
    const allowed = area === "issuer" ? ["ISSUER"] as const
      : area === "admin" ? ["ADMIN", "COMPLIANCE", "OPERATIONS"] as const
      : investorPaths.has(area) ? ["INVESTOR"] as const : null;
    if (allowed && hasAllowedRole(user, allowed)) return url.pathname + url.search + url.hash;
  } catch { /* Invalid redirect falls back to the account's workspace. */ }
  return home;
}


export function workspacesForUser(user: AuthUser | null): { label: string; href: string }[] {
  return [
    { label: "Investor workspace", href: "/dashboard", roles: ["INVESTOR"] as const },
    { label: "Issuer workspace", href: "/issuer", roles: ["ISSUER"] as const },
    { label: "Admin workspace", href: "/admin", roles: ["ADMIN", "COMPLIANCE", "OPERATIONS"] as const },
  ].filter((workspace) => hasAllowedRole(user, workspace.roles))
    .map(({ label, href }) => ({ label, href }));
}
