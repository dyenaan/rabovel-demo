"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { mockDelay } from "@/lib/api/mock-delay";
import { queryKeys } from "@/lib/query-keys";
import { mockWallets } from "@/mocks/custody.mock";
import type { Blockchain } from "@/types";
import { getWallets } from "../api/get-wallets";

export function useWallets() {
  return useQuery({
    queryKey: queryKeys.custody.wallets(),
    queryFn: getWallets,
    staleTime: QUERY_STALE_TIME.medium,
  });
}

export function useConnectWallet() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (input: { label: string; address: string; blockchain: Blockchain }) => {
      await mockDelay(700);
      mockWallets.push({
        walletId: `wal_${Date.now()}`,
        label: input.label,
        address: input.address,
        blockchain: input.blockchain,
        walletType: "SELF_CUSTODY",
        status: "PENDING_VERIFICATION",
        isPrimary: false,
        linkedAt: new Date().toISOString(),
      });
      return mockWallets;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.custody.wallets() });
      toast.success("Wallet binding submitted for verification.");
    },
  });
}
