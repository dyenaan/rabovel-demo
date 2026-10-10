import { create } from "zustand";
import type { AuthUser } from "@/features/auth/types/auth.types";

type AuthState = {
  user: AuthUser | null;
  hasHydrated: boolean;
  sessionValidated: boolean;
  setSession: (user: AuthUser) => void;
  clearSession: () => void;
  setHasHydrated: (value: boolean) => void;
  setSessionValidated: (value: boolean) => void;
};

// The HttpOnly cookie is the credential. Browser state is an in-memory UI view.
export const useAuthStore = create<AuthState>()((set) => ({
  user: null,
  hasHydrated: false,
  sessionValidated: false,
  setSession: (user) => set({ user, sessionValidated: true }),
  clearSession: () => set({ user: null, sessionValidated: true }),
  setHasHydrated: (hasHydrated) => set({ hasHydrated }),
  setSessionValidated: (sessionValidated) => set({ sessionValidated }),
}));
