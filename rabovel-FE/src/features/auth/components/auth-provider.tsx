"use client";

import { type ReactNode, useEffect } from "react";
import { useMockApi } from "@/lib/env";
import { useAuthStore } from "@/stores/auth-store";
import { fetchCurrentUser } from "../api/backend-auth";

export function AuthProvider({ children }: { children: ReactNode }) {
  useEffect(() => {
    let active = true;
    // Remove credentials persisted by older releases, even if no cookie exists.
    try { localStorage.removeItem("rabovel-auth"); } catch { /* Storage can be disabled. */ }
    const { setHasHydrated, setSession, clearSession, setSessionValidated } = useAuthStore.getState();
    setHasHydrated(true);
    if (useMockApi) {
      setSessionValidated(true);
    } else {
      void fetchCurrentUser()
        .then((user) => { if (active && !useAuthStore.getState().sessionValidated) setSession(user); })
        .catch(() => { if (active && !useAuthStore.getState().sessionValidated) clearSession(); });
    }
    return () => { active = false; };
  }, []);

  return <>{children}</>;
}
