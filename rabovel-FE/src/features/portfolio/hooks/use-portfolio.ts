"use client";

import { useQuery } from "@tanstack/react-query";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { isMockApiEnabled } from "@/lib/api/client";
import { Decimal } from "@/lib/formatters";
import { queryKeys } from "@/lib/query-keys";
import { getInvestorCatalog } from "@/features/assets/api/get-assets";
import { useAuthStore } from "@/stores/auth-store";
import type { Holding, InvestorCatalog, PortfolioSummary } from "@/types";
import {
  getHoldings,
  getPortfolioActivity,
  getPortfolioPerformance,
  getPortfolioSummary,
} from "../api/get-portfolio";

export function usePortfolioSummary() {
  const token = useAuthStore((state) => state.token);
  const legacy = useQuery({
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
    enabled: isMockApiEnabled,
  });
  const catalog = useQuery({
    queryKey: ["investor", "catalog"],
    queryFn: () => getInvestorCatalog(token!),
    enabled: !isMockApiEnabled && Boolean(token),
    staleTime: QUERY_STALE_TIME.short,
  });

  if (isMockApiEnabled) return legacy;
  return {
    data: catalog.data ? portfolioSummaryFromCatalog(catalog.data) : undefined,
    isPending: Boolean(token) && catalog.isPending,
  };
}

export function useHoldings() {
  const token = useAuthStore((state) => state.token);
  const legacy = useQuery({
    queryKey: queryKeys.portfolio.holdings(),
    queryFn: getHoldings,
    placeholderData: [],
    staleTime: QUERY_STALE_TIME.short,
    enabled: isMockApiEnabled,
  });
  const catalog = useQuery({
    queryKey: ["investor", "catalog"],
    queryFn: () => getInvestorCatalog(token!),
    enabled: !isMockApiEnabled && Boolean(token),
    staleTime: QUERY_STALE_TIME.short,
  });

  if (isMockApiEnabled) return legacy;
  return {
    data: catalog.data ? holdingsFromCatalog(catalog.data) : [],
    isPending: Boolean(token) && catalog.isPending,
  };
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

function portfolioSummaryFromCatalog(catalog: InvestorCatalog): PortfolioSummary {
  const holdings = holdingsFromCatalog(catalog);
  const totalValue = holdings.reduce(
    (total, holding) => total.plus(holding.marketValue),
    new Decimal(0),
  );
  const cashBalance = fromBaseUnits(
    catalog.cngn.balance_base_units ?? "0",
    catalog.cngn.decimals ?? 6,
  );

  return {
    totalValue: totalValue.toString(),
    totalCostBasis: totalValue.toString(),
    totalUnrealizedPnl: "0",
    totalUnrealizedPnlPercent: "0",
    cashBalance: cashBalance.toString(),
    investedValue: totalValue.toString(),
    ytdIncome: "0",
  };
}

function holdingsFromCatalog(catalog: InvestorCatalog): Holding[] {
  const positions = catalog.assets
    .filter((asset) => new Decimal(asset.investor_balance || "0").gt(0))
    .map((asset) => {
      const quantity = new Decimal(asset.investor_balance);
      const unitPrice = fromBaseUnits(asset.price_per_unit, asset.price_decimals);
      const marketValue = quantity.times(unitPrice);
      return { asset, quantity, unitPrice, marketValue };
    });
  const totalValue = positions.reduce(
    (total, position) => total.plus(position.marketValue),
    new Decimal(0),
  );

  return positions.map(({ asset, quantity, unitPrice, marketValue }) => ({
    assetId: asset.asset_id,
    assetName: asset.name,
    symbol: asset.ticker,
    quantity: quantity.toString(),
    averageCost: unitPrice.toString(),
    marketValue: marketValue.toString(),
    costBasis: marketValue.toString(),
    unrealizedPnl: "0",
    unrealizedPnlPercent: "0",
    allocationPercent: totalValue.gt(0)
      ? marketValue.dividedBy(totalValue).times(100).toString()
      : "0",
  }));
}

function fromBaseUnits(value: string, decimals: number): Decimal {
  return new Decimal(value || "0").dividedBy(new Decimal(10).pow(decimals));
}
