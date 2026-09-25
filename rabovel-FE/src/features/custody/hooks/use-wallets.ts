"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { mockDelay } from "@/lib/api/mock-delay";
import { queryKeys } from "@/lib/query-keys";
import { useMockApi } from "@/lib/env";
import { mockWallets } from "@/mocks/custody.mock";
import { useAuthStore } from "@/stores/auth-store";
import { completeSolanaWalletLink, createSolanaChallenge, getWallets } from "../api/get-wallets";

type PhantomProvider = {
  isPhantom?: boolean;
  connect: () => Promise<{ publicKey: { toString: () => string } }>;
  signMessage: (message: Uint8Array, display?: "utf8") => Promise<{ signature: Uint8Array }>;
};

function phantomProvider(): PhantomProvider {
  const browser = window as Window & { phantom?: { solana?: PhantomProvider }; solana?: PhantomProvider };
  const provider = browser.phantom?.solana ?? browser.solana;
  if (!provider?.isPhantom) throw new Error("Phantom is not installed. Install or enable Phantom, then try again.");
  return provider;
}

function encodeBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return window.btoa(binary);
}

export function useWallets() {
  const token = useAuthStore((state) => state.token);
  return useQuery({
    queryKey: queryKeys.custody.wallets(),
    queryFn: () => getWallets(token!),
    enabled: Boolean(token),
    staleTime: QUERY_STALE_TIME.medium,
  });
}

export function useConnectWallet() {
  const queryClient = useQueryClient();
  const token = useAuthStore((state) => state.token);
  return useMutation({
    mutationFn: async () => {
      if (!token) throw new Error("Sign in before linking a wallet.");
      const provider = phantomProvider();
      const connection = await provider.connect();
      const address = connection.publicKey.toString();
      if (useMockApi) {
        await mockDelay(300);
        const wallet = { walletId: `wal_${Date.now()}`, label: "Phantom wallet", address, blockchain: "SOLANA" as const, walletType: "SELF_CUSTODY" as const, status: "ACTIVE" as const, isPrimary: mockWallets.length === 0, linkedAt: new Date().toISOString() };
        mockWallets.push(wallet);
        return wallet;
      }
      const challenge = await createSolanaChallenge(token, address);
      const signed = await provider.signMessage(new TextEncoder().encode(challenge.message), "utf8");
      return completeSolanaWalletLink(token, challenge.challenge_id, encodeBase64(signed.signature));
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.custody.wallets() });
      queryClient.invalidateQueries({ queryKey: ["investor", "catalog"] });
      toast.success("Phantom wallet verified and linked.");
    },
  });
}
