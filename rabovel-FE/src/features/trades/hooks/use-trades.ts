"use client";

import { useQuery } from "@tanstack/react-query";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { queryKeys } from "@/lib/query-keys";
import { getTrades } from "../api/get-trades";

export function useTrades() {
  return useQuery({
    queryKey: queryKeys.trades.all,
    queryFn: getTrades,
    staleTime: QUERY_STALE_TIME.short,
  });
}
