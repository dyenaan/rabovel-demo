import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import {
  getMockOrderBook,
  getMockPriceHistory,
  getMockRecentTrades,
  mockMarkets,
} from "@/mocks/markets.mock";
import type { Market, OrderBookSnapshot, PricePoint, PublicTrade } from "@/types";

export async function getMarkets(): Promise<Market[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockMarkets;
  }
  const { data } = await apiClient.get<Market[]>("/markets");
  return data;
}

export async function getMarket(marketId: string): Promise<Market | undefined> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockMarkets.find((market) => market.marketId === marketId);
  }
  const { data } = await apiClient.get<Market>(`/markets/${marketId}`);
  return data;
}

export async function getOrderBook(marketId: string): Promise<OrderBookSnapshot> {
  if (isMockApiEnabled) {
    await mockDelay(200);
    return getMockOrderBook(marketId);
  }
  const { data } = await apiClient.get<OrderBookSnapshot>(`/markets/${marketId}/order-book`);
  return data;
}

export async function getPriceHistory(marketId: string, range: string): Promise<PricePoint[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    const points = range === "1D" ? 24 : range === "1W" ? 168 : 120;
    return getMockPriceHistory(marketId, points);
  }
  const { data } = await apiClient.get<PricePoint[]>(`/markets/${marketId}/price-history`, {
    params: { range },
  });
  return data;
}

export async function getRecentTrades(marketId: string): Promise<PublicTrade[]> {
  if (isMockApiEnabled) {
    await mockDelay(200);
    return getMockRecentTrades(marketId);
  }
  const { data } = await apiClient.get<PublicTrade[]>(`/markets/${marketId}/trades`);
  return data;
}
