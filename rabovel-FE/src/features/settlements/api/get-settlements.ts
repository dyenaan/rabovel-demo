import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockSettlements } from "@/mocks/settlements.mock";
import type { Settlement } from "@/types";

export async function getSettlements(): Promise<Settlement[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockSettlements;
  }
  const { data } = await apiClient.get<Settlement[]>("/settlements");
  return data;
}

export async function getSettlement(settlementId: string): Promise<Settlement | undefined> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockSettlements.find((s) => s.settlementId === settlementId);
  }
  const { data } = await apiClient.get<Settlement>(`/settlements/${settlementId}`);
  return data;
}
