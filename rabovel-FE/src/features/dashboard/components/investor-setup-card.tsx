"use client";

import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import { ArrowRight, Check, Coins, ShoppingBag, Wallet } from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ConnectWalletDialog } from "@/features/custody/components/connect-wallet-dialog";
import { getInvestorCngnStatus } from "@/features/custody/api/get-wallets";
import { useWallets } from "@/features/custody/hooks/use-wallets";
import { QUERY_STALE_TIME } from "@/lib/constants";
import { useMockApi } from "@/lib/env";
import { useAuthStore } from "@/stores/auth-store";

export function InvestorSetupCard() {
  const token = useAuthStore((state) => state.token);
  const wallets = useWallets();
  const hasSolanaWallet = Boolean(wallets.data?.some((wallet) => wallet.blockchain === "SOLANA"));
  const cngn = useQuery({
    queryKey: ["custody", "cngn"],
    queryFn: () => getInvestorCngnStatus(token!),
    enabled: !useMockApi && Boolean(token) && hasSolanaWallet,
    staleTime: QUERY_STALE_TIME.short,
    refetchOnMount: "always",
  });
  const hasCngnAccount = useMockApi ? hasSolanaWallet : Boolean(cngn.data?.account_verified);
  const hasFunds = useMockApi
    ? hasCngnAccount
    : Boolean(cngn.data?.balance_base_units && !/^0+$/.test(cngn.data.balance_base_units));
  const complete = hasSolanaWallet && hasCngnAccount && hasFunds;

  return (
    <Card className={complete ? "border-success/30 bg-success/[0.03]" : "border-primary/25 bg-primary/[0.03]"}>
      <CardHeader>
        <CardTitle>{complete ? "You’re ready to invest" : "Get ready to invest"}</CardTitle>
        <CardDescription>
          Connect your Phantom wallet, add cNGN as your payment method, then purchase any live equity.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-5">
        <ol className="grid gap-3 md:grid-cols-3">
          <Step number={1} title="Connect wallet" description="Verify ownership with Phantom." complete={hasSolanaWallet} icon={Wallet} />
          <Step number={2} title="Fund with cNGN" description="Create the account and add demo cNGN." complete={hasFunds} icon={Coins} />
          <Step number={3} title="Buy an asset" description="Review a quote and settle on-chain." complete={false} icon={ShoppingBag} />
        </ol>

        <div className="flex flex-wrap items-center gap-3">
          {!hasSolanaWallet ? (
            <ConnectWalletDialog label="Connect Phantom wallet" />
          ) : !hasFunds ? (
            <Button asChild>
              <Link href="/wallet">Set up and fund cNGN <ArrowRight /></Link>
            </Button>
          ) : (
            <Button asChild>
              <Link href="/assets">Browse live assets <ArrowRight /></Link>
            </Button>
          )}
          {!wallets.isPending && wallets.isError && (
            <p role="alert" className="text-sm text-destructive">Could not check your linked wallets.</p>
          )}
          {hasSolanaWallet && !useMockApi && cngn.isError && (
            <p role="alert" className="text-sm text-destructive">Could not check your cNGN account. Open Wallet to retry.</p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function Step({ number, title, description, complete, icon: Icon }: {
  number: number;
  title: string;
  description: string;
  complete: boolean;
  icon: LucideIcon;
}) {
  return (
    <li className="flex gap-3 rounded-lg border bg-background/80 p-4">
      <span className={`flex size-9 shrink-0 items-center justify-center rounded-full ${complete ? "bg-success text-white" : "bg-muted text-muted-foreground"}`}>
        {complete ? <Check className="size-4" /> : <Icon className="size-4" />}
      </span>
      <div className="min-w-0">
        <p className="flex items-center gap-2 text-sm font-medium"><span className="text-xs text-muted-foreground">{number}</span>{title}</p>
        <p className="mt-1 text-xs text-muted-foreground">{description}</p>
      </div>
    </li>
  );
}
