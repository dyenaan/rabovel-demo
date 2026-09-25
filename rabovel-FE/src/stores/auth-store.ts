import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AuthUser } from "@/features/auth/types/auth.types";

type AuthState = {
  user: AuthUser | null;
  token: string | null;
  hasHydrated: boolean;
  sessionValidated: boolean;
  setSession: (user: AuthUser, token: string) => void;
  clearSession: () => void;
  setHasHydrated: (value: boolean) => void;
  setSessionValidated: (value: boolean) => void;
};

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      hasHydrated: false,
      sessionValidated: false,
      setSession: (user, token) => set({ user, token, sessionValidated: true }),
      clearSession: () => set({ user: null, token: null, sessionValidated: true }),
      setHasHydrated: (value) => set({ hasHydrated: value }),
      setSessionValidated: (value) => set({ sessionValidated: value }),
    }),
    {
      name: "rabovel-auth",
      partialize: (state) => ({ user: state.user, token: state.token }),
      onRehydrateStorage: () => (state) => state?.setHasHydrated(true),
    },
  ),
);
