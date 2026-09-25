"use client";

import { useEffect, useState } from "react";
import { Check, Copy, RefreshCw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { createBrokerCngnAccount, fetchCngnPaymentAsset, type PaymentAssetStatus } from "@/features/auth/api/backend-auth";
import { Decimal, formatMoney } from "@/lib/formatters";
import { useAuthStore } from "@/stores/auth-store";

export function PaymentAssetStatusCard() {
  const token = useAuthStore((state) => state.token);
  const [status, setStatus] = useState<PaymentAssetStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isCreating, setIsCreating] = useState(false);

  async function loadStatus() {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      setStatus(await fetchCngnPaymentAsset(token));
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : "Could not load the cNGN payment rail.");
    } finally {
      setIsLoading(false);
    }
  }

  async function createAccount() {
    if (!token) return;
    setIsCreating(true);
    setError(null);
    try {
      setStatus(await createBrokerCngnAccount(token));
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : "Could not create the issuer settlement cNGN account.");
    } finally {
      setIsCreating(false);
    }
  }

  useEffect(() => {
    if (!token) return;
    let active = true;
    void fetchCngnPaymentAsset(token)
      .then((paymentAsset) => {
        if (active) setStatus(paymentAsset);
      })
      .catch((requestError: unknown) => {
        if (active) setError(requestError instanceof Error ? requestError.message : "Could not load the cNGN payment rail.");
      })
      .finally(() => {
        if (active) setIsLoading(false);
      });
    return () => { active = false; };
  }, [token]);

  return <Card className="md:col-span-2"><CardHeader><div className="flex flex-wrap items-start justify-between gap-3"><div><CardTitle>CNGN payment rail</CardTitle><CardDescription className="mt-1">Fund the Admin wallet with test SOL, create the issuer settlement wallet’s associated cNGN account, then fund that token-account address from the faucet.</CardDescription></div><div className="flex flex-wrap items-center gap-2"><Badge variant={status?.ready ? "success" : "warning"}>{status?.ready ? "Ready" : "Not ready"}</Badge>{status && !status.broker_account_verified && <Button type="button" size="sm" disabled={isCreating || isLoading || !status.mint_verified} onClick={() => void createAccount()}>{isCreating && <RefreshCw className="animate-spin" />}{isCreating ? "Creating…" : "Create settlement cNGN account"}</Button>}<Button type="button" variant="outline" size="sm" disabled={isLoading || isCreating || !token} onClick={() => void loadStatus()}><RefreshCw className={isLoading ? "animate-spin" : undefined} />Refresh</Button></div></div></CardHeader><CardContent className="grid gap-4 text-sm sm:grid-cols-2"><Detail label="Network" value={status?.network ?? (isLoading ? "Loading…" : "Unavailable")} /><Detail label="Token program" value={status?.token_program ?? "Unverified"} /><Detail label="Decimals" value={status?.decimals?.toString() ?? "Unverified"} /><Detail label="Issuer cNGN balance" value={displayNairaBalance(status)} /><AddressDetail label="Admin fee-payer wallet" address={status?.fee_payer_address} hint="Send test SOL here. The backend uses it to pay transaction and account-creation fees." /><AddressDetail label="Issuer settlement wallet (owner)" address={status?.broker_owner_address} hint="This is the Solana keypair’s public address and owns the token account below." /><AddressDetail label="Issuer settlement cNGN token account" address={status?.broker_token_account} hint="Use this address as the faucet recipient once the account is created." /><AddressDetail label="cNGN mint" address={status?.mint_address} />{error && <p role="alert" className="sm:col-span-2 text-destructive">{error}</p>}{status?.error && <p className="sm:col-span-2 text-amber-700 dark:text-amber-300">{status.error}</p>}</CardContent></Card>;
}

function displayNairaBalance(status: PaymentAssetStatus | null): string {
  if (status?.broker_balance_base_units === null || status?.broker_balance_base_units === undefined || status.decimals === null) {
    return "Unverified";
  }
  const naira = new Decimal(status.broker_balance_base_units)
    .dividedBy(new Decimal(10).pow(status.decimals));
  return `${formatMoney(naira.toString())} cNGN`;
}

function Detail({ label, value }: { label: string; value: string }) {
  return <div><p className="text-xs text-muted-foreground">{label}</p><p className="mt-1 break-all font-medium">{value}</p></div>;
}

function AddressDetail({ label, address, hint }: { label: string; address: string | null | undefined; hint?: string }) {
  const [copied, setCopied] = useState(false);

  async function copyAddress() {
    if (!address) return;
    await navigator.clipboard.writeText(address);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1500);
  }

  return <div className="sm:col-span-2"><p className="text-xs text-muted-foreground">{label}</p><div className="mt-1 flex items-start gap-2"><p className="min-w-0 flex-1 break-all font-mono text-xs">{address ?? "Not available"}</p>{address && <Button type="button" variant="ghost" size="icon" className="-mt-1 size-7 shrink-0" onClick={() => void copyAddress()} aria-label={`Copy ${label.toLowerCase()}`}>{copied ? <Check className="text-success" /> : <Copy />}</Button>}</div>{hint && <p className="mt-1 text-xs text-muted-foreground">{hint}</p>}</div>;
}
