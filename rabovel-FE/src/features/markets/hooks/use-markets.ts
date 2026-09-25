"use client";

import { useQuery } from "@tanstack/react-query";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { queryKeys } from "@/lib/query-keys";
import { useMarketStream, useOrderBookStream, useTradeStream } from "@/lib/websocket/hooks";
import type { Market } from "@/types";
import {
  getMarket,
  getMarkets,
  getOrderBook,
  getPriceHistory,
  getRecentTrades,
} from "../api/get-markets";

export function useMarkets() {
  return useQuery({
    queryKey: queryKeys.markets.all,
    queryFn: getMarkets,
    staleTime: QUERY_STALE_TIME.short,
  });
}

export function useMarket(marketId: string) {
  const query = useQuery({
    queryKey: queryKeys.markets.detail(marketId),
    queryFn: () => getMarket(marketId),
    staleTime: QUERY_STALE_TIME.short,
    enabled: !!marketId,
  });

  const live = useMarketStream(marketId, query.data);

  return { ...query, data: (live ?? query.data) as Market | undefined };
}

export function useOrderBook(marketId: string) {
  const query = useQuery({
    queryKey: queryKeys.markets.orderBook(marketId),
    queryFn: () => getOrderBook(marketId),
    staleTime: QUERY_STALE_TIME.realtime,
    enabled: !!marketId,
  });

  const live = useOrderBookStream(marketId, query.data);

  return { ...query, data: live ?? query.data };
}

export function usePriceHistory(marketId: string, range: string) {
  return useQuery({
    queryKey: queryKeys.markets.priceHistory(marketId, range),
    queryFn: () => getPriceHistory(marketId, range),
    staleTime: QUERY_STALE_TIME.medium,
    enabled: !!marketId,
  });
}

export function useRecentTrades(marketId: string) {
  const query = useQuery({
    queryKey: queryKeys.markets.recentTrades(marketId),
    queryFn: () => getRecentTrades(marketId),
    staleTime: QUERY_STALE_TIME.realtime,
    enabled: !!marketId,
  });

  const live = useTradeStream(marketId, query.data ?? []);

  return { ...query, data: live.length > 0 ? live : query.data };
}
