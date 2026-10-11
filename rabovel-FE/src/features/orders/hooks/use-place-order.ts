"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import { queryKeys } from "@/lib/query-keys";
import { cancelOrder, placeOrder, type PlaceOrderInput } from "../api/place-order";

export function usePlaceOrder() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: PlaceOrderInput) => placeOrder(input),
    onSuccess: (order) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.orders.all });
      void queryClient.invalidateQueries({ queryKey: queryKeys.portfolio.all });
      void queryClient.invalidateQueries({ queryKey: queryKeys.markets.all });
      toast.success(`Order placed — ${order.side} ${order.quantity} ${order.marketSymbol}`);
    },
    onError: (error) => {
      toast.error(error instanceof Error ? error.message : "Failed to place order.");
    },
  });
}

export function useCancelOrder() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ orderId, idempotencyKey }: { orderId: string; idempotencyKey: string }) =>
      cancelOrder(orderId, idempotencyKey),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.orders.all });
      void queryClient.invalidateQueries({ queryKey: queryKeys.portfolio.all });
      void queryClient.invalidateQueries({ queryKey: queryKeys.markets.all });
      toast.success("Order cancelled.");
    },
    onError: (error) => {
      toast.error(error instanceof Error ? error.message : "Failed to cancel order.");
    },
  });
}
