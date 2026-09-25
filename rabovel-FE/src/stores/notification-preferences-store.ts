import { create } from "zustand";
import { persist } from "zustand/middleware";

type NotificationPreferencesState = {
  emailAlerts: boolean;
  orderFillAlerts: boolean;
  settlementAlerts: boolean;
  complianceAlerts: boolean;
  marketingUpdates: boolean;
  setPreference: (
    key: Exclude<keyof NotificationPreferencesState, "setPreference">,
    value: boolean,
  ) => void;
};

export const useNotificationPreferencesStore = create<NotificationPreferencesState>()(
  persist(
    (set) => ({
      emailAlerts: true,
      orderFillAlerts: true,
      settlementAlerts: true,
      complianceAlerts: true,
      marketingUpdates: false,
      setPreference: (key, value) => set({ [key]: value }),
    }),
    { name: "rabovel-notification-preferences" },
  ),
);
