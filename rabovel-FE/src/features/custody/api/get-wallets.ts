import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockWallets } from "@/mocks/custody.mock";
import type { Wallet } from "@/types";

export async function getWallets(): Promise<Wallet[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockWallets;
  }
  const { data } = await apiClient.get<Wallet[]>("/custody/wallets");
  return data;
}
