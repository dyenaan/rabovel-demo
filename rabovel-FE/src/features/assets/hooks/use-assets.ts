"use client";

import { useQuery } from "@tanstack/react-query";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { queryKeys } from "@/lib/query-keys";
import { getAsset, getAssetNavHistory, getAssets } from "../api/get-assets";

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
