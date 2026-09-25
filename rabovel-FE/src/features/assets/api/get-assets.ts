import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockAssets, mockNavHistory } from "@/mocks/assets.mock";
import type { Asset, InvestorCatalog, InvestorQuote, NavHistoryPoint } from "@/types";
import { env } from "@/lib/env";

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

export async function getInvestorCatalog(token: string): Promise<InvestorCatalog> {
  return investorRequest<InvestorCatalog>("/investor/catalog", token);
}

export async function createInvestorQuote(
  token: string,
  assetId: string,
  quantity: string,
): Promise<InvestorQuote> {
  return investorRequest<InvestorQuote>("/investor/quote", token, {
    method: "POST",
    body: JSON.stringify({ asset_id: assetId, side: "buy", quantity }),
  });
}

async function investorRequest<T>(path: string, token: string, init?: RequestInit): Promise<T> {
  const response = await fetch(new URL(path, env.NEXT_PUBLIC_API_BASE_URL), {
    ...init,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
      ...init?.headers,
    },
  });
  const payload = (await response.json().catch(() => null)) as
    | T
    | { error?: { message?: string }; message?: string }
    | null;
  if (!response.ok) {
    const details = payload as { error?: { message?: string }; message?: string } | null;
    throw new Error(
      details?.error?.message ?? details?.message ?? "Could not complete the investor request.",
    );
  }
  return payload as T;
}
