"use client";

import { type ReactNode, useEffect } from "react";
import { registerAuthTokenProvider } from "@/lib/api/client";
import { useMockApi } from "@/lib/env";
import { useAuthStore } from "@/stores/auth-store";
import { fetchCurrentUser } from "../api/backend-auth";

export function AuthProvider({ children }: { children: ReactNode }) {
  const hasHydrated = useAuthStore((state) => state.hasHydrated);

  useEffect(() => {
    registerAuthTokenProvider(() => useAuthStore.getState().token ?? undefined);
  }, []);

  useEffect(() => {
    if (!hasHydrated) return;
    const { token, clearSession, setSession, setSessionValidated } = useAuthStore.getState();
    if (useMockApi) {
      if (token && !token.startsWith("mock_token_")) clearSession();
      else setSessionValidated(true);
      return;
    }
    if (!token || token.startsWith("mock_token_")) {
      clearSession();
      return;
    }
    setSessionValidated(false);
    void fetchCurrentUser(token)
      .then((user) => setSession(user, token))
      .catch(() => clearSession());
  }, [hasHydrated]);

  return <>{children}</>;
}
