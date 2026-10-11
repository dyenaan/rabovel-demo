"use client";

import { useQuery } from "@tanstack/react-query";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { livePollInterval } from "@/lib/api/client";
import { queryKeys } from "@/lib/query-keys";
import {
  getHoldings,
  getPortfolioActivity,
  getPortfolioPerformance,
  getPortfolioSummary,
} from "../api/get-portfolio";

// Real mode: engine trades plus wallet-held issuer investments, with wallet
// cNGN counted once as cash (see the backend /portfolio/* routes).

export function usePortfolioSummary() {
  return useQuery({
    queryKey: queryKeys.portfolio.summary(),
    queryFn: getPortfolioSummary,
    staleTime: QUERY_STALE_TIME.short,
    refetchInterval: livePollInterval,
  });
}

export function useHoldings() {
  return useQuery({
    queryKey: queryKeys.portfolio.holdings(),
    queryFn: getHoldings,
    placeholderData: [],
    staleTime: QUERY_STALE_TIME.short,
    refetchInterval: livePollInterval,
  });
}

export function usePortfolioPerformance(range: string) {
  return useQuery({
    queryKey: queryKeys.portfolio.performance(range),
    queryFn: () => getPortfolioPerformance(range),
    placeholderData: [],
    staleTime: QUERY_STALE_TIME.medium,
    refetchInterval: livePollInterval,
  });
}

export function usePortfolioActivity() {
  return useQuery({
    queryKey: queryKeys.portfolio.activity(),
    queryFn: getPortfolioActivity,
    placeholderData: [],
    staleTime: QUERY_STALE_TIME.short,
    refetchInterval: livePollInterval,
  });
}
