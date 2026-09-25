"use client";

import { useQuery } from "@tanstack/react-query";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { queryKeys } from "@/lib/query-keys";
import { getSettlement, getSettlements } from "../api/get-settlements";

export function useSettlements() {
  return useQuery({
    queryKey: queryKeys.settlements.all,
    queryFn: getSettlements,
    staleTime: QUERY_STALE_TIME.short,
  });
}

export function useSettlement(settlementId: string) {
  return useQuery({
    queryKey: queryKeys.settlements.detail(settlementId),
    queryFn: () => getSettlement(settlementId),
    staleTime: QUERY_STALE_TIME.short,
    enabled: !!settlementId,
  });
}
