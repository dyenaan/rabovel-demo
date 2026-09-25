"use client";

import { Wallet as WalletIcon } from "lucide-react";

import { EmptyState } from "@/components/shared/empty-state";
import { LoadingState } from "@/components/shared/loading-state";
import { useWallets } from "../hooks/use-wallets";
import { WalletCard } from "./wallet-card";

export function WalletBindingList() {
  const { data, isPending } = useWallets();

  if (isPending) return <LoadingState label="Loading wallets…" />;
  if (!data || data.length === 0) {
    return (
      <EmptyState
        icon={WalletIcon}
        title="No wallets bound"
        description="Bind a custody wallet to receive settlement of tokenized assets."
      />
    );
  }

  return (
    <div className="space-y-3">
      {data.map((wallet) => (
        <WalletCard key={wallet.walletId} wallet={wallet} />
      ))}
    </div>
  );
}
