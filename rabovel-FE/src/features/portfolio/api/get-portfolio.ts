import { apiClient, isMockApiEnabled } from "@/lib/api/client";
import { mockDelay } from "@/lib/api/mock-delay";
import {
  mockActivity,
  mockHoldings,
  mockPerformanceHistory,
  mockPortfolioSummary,
} from "@/mocks/portfolio.mock";
import type { ActivityEvent, Holding, PerformancePoint, PortfolioSummary } from "@/types";

export async function getPortfolioSummary(): Promise<PortfolioSummary> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockPortfolioSummary;
  }
  const { data } = await apiClient.get<PortfolioSummary>("/portfolio/summary");
  return data;
}

export async function getHoldings(): Promise<Holding[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockHoldings;
  }
  const { data } = await apiClient.get<Holding[]>("/portfolio/holdings");
  return data;
}

export async function getPortfolioPerformance(range: string): Promise<PerformancePoint[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    const days = range === "1M" ? 30 : range === "3M" ? 90 : range === "1Y" ? 180 : 180;
    return mockPerformanceHistory.slice(-days);
  }
  const { data } = await apiClient.get<PerformancePoint[]>("/portfolio/performance", {
    params: { range },
  });
  return data;
}

export async function getPortfolioActivity(): Promise<ActivityEvent[]> {
  if (isMockApiEnabled) {
    await mockDelay();
    return mockActivity;
  }
  const { data } = await apiClient.get<ActivityEvent[]>("/portfolio/activity");
  return data;
}
