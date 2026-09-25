"use client";

import { useQuery } from "@tanstack/react-query";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { queryKeys } from "@/lib/query-keys";
import {
  getHoldings,
  getPortfolioActivity,
  getPortfolioPerformance,
  getPortfolioSummary,
} from "../api/get-portfolio";

export function usePortfolioSummary() {
  return useQuery({
    queryKey: queryKeys.portfolio.summary(),
    queryFn: getPortfolioSummary,
    placeholderData: {
      totalValue: "0",
      totalCostBasis: "0",
      totalUnrealizedPnl: "0",
      totalUnrealizedPnlPercent: "0",
      cashBalance: "0",
      investedValue: "0",
      ytdIncome: "0",
    },
    staleTime: QUERY_STALE_TIME.short,
  });
}

export function useHoldings() {
  return useQuery({
    queryKey: queryKeys.portfolio.holdings(),
    queryFn: getHoldings,
    placeholderData: [],
    staleTime: QUERY_STALE_TIME.short,
  });
}

export function usePortfolioPerformance(range: string) {
  return useQuery({
    queryKey: queryKeys.portfolio.performance(range),
    queryFn: () => getPortfolioPerformance(range),
    placeholderData: [],
    staleTime: QUERY_STALE_TIME.medium,
  });
}

export function usePortfolioActivity() {
  return useQuery({
    queryKey: queryKeys.portfolio.activity(),
    queryFn: getPortfolioActivity,
    placeholderData: [],
    staleTime: QUERY_STALE_TIME.short,
  });
}
