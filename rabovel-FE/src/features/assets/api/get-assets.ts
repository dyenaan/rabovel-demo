import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockAssets, mockNavHistory } from "@/mocks/assets.mock";
import type { Asset, InvestorCatalog, InvestorQuote, NavHistoryPoint, PreparedPurchase, PurchaseSettlement } from "@/types";
import { gatewayFetch } from "@/lib/api/gateway-fetch";

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

export async function getInvestorCatalog(): Promise<InvestorCatalog> {
  return investorRequest<InvestorCatalog>("/investor/catalog");
}

export async function createInvestorQuote(
  assetId: string,
  quantity: string,
): Promise<InvestorQuote> {
  return investorRequest<InvestorQuote>("/investor/quote", {
    method: "POST",
    body: JSON.stringify({ asset_id: assetId, side: "buy", quantity }),
  });
}

export async function prepareInvestorPurchase(assetId: string, quantity: string) {
  return investorRequest<PreparedPurchase>("/investor/purchase/prepare", {
    method: "POST",
    body: JSON.stringify({ asset_id: assetId, quantity }),
  });
}

export async function submitInvestorPurchase(transactionBase64: string) {
  return investorRequest<PurchaseSettlement>("/investor/purchase/submit", {
    method: "POST",
    body: JSON.stringify({ transaction_base64: transactionBase64 }),
  });
}

async function investorRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await gatewayFetch(path, {
    ...init,
    headers: {
      "Content-Type": "application/json",
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
