"use client";

import { useMutation, useQuery } from "@tanstack/react-query";
import { toast } from "sonner";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { queryKeys } from "@/lib/query-keys";
import {
  createInvestorQuote,
  getAsset,
  getAssetNavHistory,
  getAssets,
  getInvestorCatalog,
} from "../api/get-assets";
import { useAuthStore } from "@/stores/auth-store";

export function useAssets() {
  return useQuery({
    queryKey: queryKeys.assets.all,
    queryFn: getAssets,
    staleTime: QUERY_STALE_TIME.medium,
  });
}

export function useAsset(assetId: string) {
  return useQuery({
    queryKey: queryKeys.assets.detail(assetId),
    queryFn: () => getAsset(assetId),
    staleTime: QUERY_STALE_TIME.medium,
    enabled: !!assetId,
  });
}

export function useAssetNavHistory(assetId: string) {
  return useQuery({
    queryKey: queryKeys.assets.navHistory(assetId),
    queryFn: () => getAssetNavHistory(assetId),
    staleTime: QUERY_STALE_TIME.medium,
    enabled: !!assetId,
  });
}

export function useInvestorCatalog() {
  const token = useAuthStore((state) => state.token);
  return useQuery({ queryKey: ["investor", "catalog"], queryFn: () => getInvestorCatalog(token!), enabled: Boolean(token), staleTime: QUERY_STALE_TIME.short });
}

export function useInvestorQuote() {
  const token = useAuthStore((state) => state.token);
  return useMutation({
    mutationFn: ({ assetId, quantity }: { assetId: string; quantity: string }) => {
      if (!token) throw new Error("Your session has expired. Sign in and try again.");
      return createInvestorQuote(token, assetId, quantity);
    },
    onError: (error) => {
      toast.error(error instanceof Error ? error.message : "Could not create the quote.");
    },
  });
}
