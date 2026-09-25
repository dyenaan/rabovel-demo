"use client";

import { useAuthStore } from "@/stores/auth-store";
import { revokeSession } from "../api/backend-auth";
import { useMockApi } from "@/lib/env";

export function useAuth() {
  const user = useAuthStore((s) => s.user);
  const hasHydrated = useAuthStore((s) => s.hasHydrated);
  const sessionValidated = useAuthStore((s) => s.sessionValidated);
  const token = useAuthStore((s) => s.token);
  const setSession = useAuthStore((s) => s.setSession);
  const clearSession = useAuthStore((s) => s.clearSession);

  return {
    user,
    role: user?.role ?? null,
    isAuthenticated: !!user,
    isLoading: !hasHydrated || !sessionValidated,
    login: (userValue: NonNullable<typeof user>, token: string) => setSession(userValue, token),
    logout: async () => {
      try {
        if (!useMockApi && token) await revokeSession(token);
      } finally {
        clearSession();
      }
    },
  };
}
