import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AccountType } from "@/types";

type OnboardingState = {
  accountType: AccountType;
  fullName: string;
  jurisdiction: string;
  identityDocumentUploaded: boolean;
  accreditedInvestor: boolean;
  sourceOfFunds: string;
  custodyChoice: "CUSTODIAL" | "SELF_CUSTODY" | "";
  setField: <K extends keyof Omit<OnboardingState, "setField" | "reset">>(
    key: K,
    value: OnboardingState[K],
  ) => void;
  reset: () => void;
};

const initialState = {
  accountType: "INDIVIDUAL" as AccountType,
  fullName: "",
  jurisdiction: "",
  identityDocumentUploaded: false,
  accreditedInvestor: false,
  sourceOfFunds: "",
  custodyChoice: "" as const,
};

export const useOnboardingStore = create<OnboardingState>()(
  persist(
    (set) => ({
      ...initialState,
      setField: (key, value) => set({ [key]: value }),
      reset: () => set(initialState),
    }),
    { name: "rabovel-onboarding" },
  ),
);
