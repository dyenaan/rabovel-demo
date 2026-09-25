"use client";

import type { ReactNode } from "react";
import type { UserRole } from "@/types";
import { useAuth } from "../hooks/use-auth";
import { RequireAuth } from "./require-auth";

/**
 * UI-layer guard only — hides/shows navigation and screens per role for a
 * cleaner experience. The backend remains the source of truth for
 * authorization; never rely on this for actual access control.
 *
 * Signed-out visitors are redirected to /login (via RequireAuth); signed-in
 * visitors with an insufficient role see `fallback` instead.
 */
export function RoleGuard({
  allow,
  fallback = null,
  children,
}: {
  allow: UserRole[];
  fallback?: ReactNode;
  children: ReactNode;
}) {
  const { role } = useAuth();

  return <RequireAuth>{role && allow.includes(role) ? children : fallback}</RequireAuth>;
}
