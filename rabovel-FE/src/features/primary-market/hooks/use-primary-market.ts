"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import { QUERY_STALE_TIME } from "@/lib/constants";
import {
  getRedemptions,
  getReserveStatus,
  getSubscriptions,
  submitRedemption,
  submitSubscription,
} from "../api/primary-market";

const KEYS = {
  subscriptions: ["primary-market", "subscriptions"] as const,
  redemptions: ["primary-market", "redemptions"] as const,
  reserves: ["primary-market", "reserves"] as const,
};

export function useSubscriptions() {
  return useQuery({
    queryKey: KEYS.subscriptions,
    queryFn: getSubscriptions,
    staleTime: QUERY_STALE_TIME.medium,
  });
}

export function useRedemptions() {
  return useQuery({
    queryKey: KEYS.redemptions,
    queryFn: getRedemptions,
    staleTime: QUERY_STALE_TIME.medium,
  });
}

export function useReserveStatus() {
  return useQuery({
    queryKey: KEYS.reserves,
    queryFn: getReserveStatus,
    staleTime: QUERY_STALE_TIME.long,
  });
}

export function useSubmitSubscription() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: submitSubscription,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: KEYS.subscriptions });
      toast.success("Subscription submitted for review.");
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : "Failed to submit."),
  });
}

export function useSubmitRedemption() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: submitRedemption,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: KEYS.redemptions });
      toast.success("Redemption request submitted.");
    },
    onError: (error) => toast.error(error instanceof Error ? error.message : "Failed to submit."),
  });
}
