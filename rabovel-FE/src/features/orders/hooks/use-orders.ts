"use client";

import { useQuery } from "@tanstack/react-query";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { queryKeys } from "@/lib/query-keys";
import { getOrder, getOrders } from "../api/get-orders";

export function useOrders() {
  return useQuery({
    queryKey: queryKeys.orders.all,
    queryFn: getOrders,
    staleTime: QUERY_STALE_TIME.short,
  });
}

export function useOrder(orderId: string) {
  return useQuery({
    queryKey: queryKeys.orders.detail(orderId),
    queryFn: () => getOrder(orderId),
    staleTime: QUERY_STALE_TIME.short,
    enabled: !!orderId,
  });
}
