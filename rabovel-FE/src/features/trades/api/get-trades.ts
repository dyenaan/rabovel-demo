import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockTrades } from "@/mocks/trades.mock";
import type { Trade } from "@/types";

export async function getTrades(): Promise<Trade[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockTrades;
  }
  const { data } = await apiClient.get<Trade[]>("/trades");
  return data;
}
