import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockAssets, mockNavHistory } from "@/mocks/assets.mock";
import type { Asset, NavHistoryPoint } from "@/types";

export async function getAssets(): Promise<Asset[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockAssets;
  }
  const { data } = await apiClient.get<Asset[]>("/assets");
  return data;
}

export async function getAsset(assetId: string): Promise<Asset | undefined> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockAssets.find((asset) => asset.assetId === assetId);
  }
  const { data } = await apiClient.get<Asset>(`/assets/${assetId}`);
  return data;
}

export async function getAssetNavHistory(assetId: string): Promise<NavHistoryPoint[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockNavHistory[assetId] ?? [];
  }
  const { data } = await apiClient.get<NavHistoryPoint[]>(`/assets/${assetId}/nav-history`);
  return data;
}
