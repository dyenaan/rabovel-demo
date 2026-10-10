"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useAuthStore } from "@/stores/auth-store";
import { revokeSession } from "../api/backend-auth";
import { useMockApi } from "@/lib/env";

export function useAuth() {
  const user = useAuthStore((s) => s.user);
  const hasHydrated = useAuthStore((s) => s.hasHydrated);
  const sessionValidated = useAuthStore((s) => s.sessionValidated);
  const setSession = useAuthStore((s) => s.setSession);
  const clearSession = useAuthStore((s) => s.clearSession);
  const queries = useQueryClient();

  return {
    user,
    role: user?.role ?? null,
    isAuthenticated: !!user,
    isLoading: !hasHydrated || !sessionValidated,
    login: (value: NonNullable<typeof user>) => { queries.clear(); setSession(value); },
    logout: async () => {
      try {
        if (!useMockApi) await revokeSession();
      } finally {
        clearSession();
        queries.clear();
      }
    },
  };
}
