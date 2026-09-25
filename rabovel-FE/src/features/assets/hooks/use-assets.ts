"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Transaction } from "@solana/web3.js";

import { QUERY_STALE_TIME } from "@/lib/constants";
import { queryKeys } from "@/lib/query-keys";
import {
  createInvestorQuote,
  getAsset,
  getAssetNavHistory,
  getAssets,
  getInvestorCatalog,
  prepareInvestorPurchase,
  submitInvestorPurchase,
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

type PhantomProvider = {
  isPhantom?: boolean;
  connect: () => Promise<{ publicKey: { toString: () => string } }>;
  signTransaction: (transaction: Transaction) => Promise<Transaction>;
};

export function useInvestorPurchase() {
  const token = useAuthStore((state) => state.token);
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ assetId, quantity, expectedTotal }: { assetId: string; quantity: string; expectedTotal: string }) => {
      if (!token) throw new Error("Your session has expired. Sign in and try again.");
      const browser = window as Window & { phantom?: { solana?: PhantomProvider }; solana?: PhantomProvider };
      const provider = browser.phantom?.solana ?? browser.solana;
      if (!provider?.isPhantom) throw new Error("Phantom is not installed or enabled.");
      const prepared = await prepareInvestorPurchase(token, assetId, quantity);
      if (prepared.quote.total_payment !== expectedTotal) {
        throw new Error("The quote changed. Review a fresh quote before purchasing.");
      }
      const connection = await provider.connect();
      const transaction = Transaction.from(decodeBase64(prepared.transaction_base64));
      if (transaction.feePayer?.toBase58() !== connection.publicKey.toString()) {
        throw new Error("Phantom is connected to a different wallet than your linked Rabovel wallet.");
      }
      const signed = await provider.signTransaction(transaction);
      return submitInvestorPurchase(token, encodeBase64(signed.serialize()));
    },
    onSuccess: (settlement) => {
      queryClient.invalidateQueries({ queryKey: ["investor", "catalog"] });
      toast.success(`Purchase confirmed on-chain: ${shortSignature(settlement.signature)}`);
    },
    onError: (error) => {
      toast.error(error instanceof Error ? error.message : "Purchase settlement failed.");
    },
  });
}

function decodeBase64(value: string) {
  const binary = window.atob(value);
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

function encodeBase64(value: Uint8Array) {
  let binary = "";
  for (const byte of value) binary += String.fromCharCode(byte);
  return window.btoa(binary);
}

function shortSignature(value: string) {
  return `${value.slice(0, 6)}…${value.slice(-6)}`;
}
