"use client";

import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Check, Copy, RefreshCw, WalletCards } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useAuthStore } from "@/stores/auth-store";
import { createInvestorCngnAccount, getInvestorCngnStatus, type CngnWalletStatus } from "../api/get-wallets";

export function InvestorCngnCard() {
  const queryClient = useQueryClient();
  const token = useAuthStore((state) => state.token);
  const [status, setStatus] = useState<CngnWalletStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    if (!token) return;
    setLoading(true); setError(null);
    try { const next = await getInvestorCngnStatus(token); setStatus(next); queryClient.setQueryData(["custody", "cngn"], next); await queryClient.invalidateQueries({ queryKey: ["investor", "catalog"] }); }
    catch (reason) { setError(reason instanceof Error ? reason.message : "Could not load your cNGN account."); }
    finally { setLoading(false); }
  }

  async function createAccount() {
    if (!token) return;
    setCreating(true); setError(null);
    try { const next = await createInvestorCngnAccount(token); setStatus(next); queryClient.setQueryData(["custody", "cngn"], next); await queryClient.invalidateQueries({ queryKey: ["investor", "catalog"] }); }
    catch (reason) { setError(reason instanceof Error ? reason.message : "Could not create your cNGN account."); }
    finally { setCreating(false); }
  }

  useEffect(() => {
    if (!token) return;
    let active = true;
    void getInvestorCngnStatus(token)
      .then((result) => { if (active) { setStatus(result); queryClient.setQueryData(["custody", "cngn"], result); } })
      .catch((reason: unknown) => { if (active) setError(reason instanceof Error ? reason.message : "Link a Phantom wallet before creating a cNGN account."); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [queryClient, token]);

  return <Card>
    <CardHeader>
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div><CardTitle className="flex items-center gap-2"><WalletCards className="size-5" />Investor cNGN funding</CardTitle><CardDescription className="mt-1">Create your cNGN token account, then copy that token-account address into the devnet faucet.</CardDescription></div>
        <div className="flex items-center gap-2"><Badge variant={status?.ready ? "success" : "warning"}>{status?.ready ? "Ready" : "Not ready"}</Badge>{status && !status.account_verified && <Button size="sm" disabled={creating || loading} onClick={() => void createAccount()}>{creating && <RefreshCw className="animate-spin" />}{creating ? "Creating…" : "Create cNGN account"}</Button>}<Button size="icon" variant="outline" disabled={!token || loading || creating} onClick={() => void refresh()} aria-label="Refresh cNGN balance"><RefreshCw className={loading ? "animate-spin" : undefined} /></Button></div>
      </div>
    </CardHeader>
    <CardContent className="grid gap-4 text-sm sm:grid-cols-2">
      <Detail label="Network" value={status?.network ?? (loading ? "Loading…" : "Unavailable")} />
      <Detail label="Balance" value={displayBalance(status)} />
      <Address label="Phantom wallet" value={status?.wallet_address} />
      <Address label="cNGN faucet recipient" value={status?.token_account} hint="Send faucet tokens to this token-account address, not the wallet address above." />
      <Address label="cNGN mint" value={status?.mint_address} />
      {error && <p role="alert" className="sm:col-span-2 text-destructive">{error}</p>}
      {status?.error && <p className="sm:col-span-2 text-amber-700 dark:text-amber-300">{status.error}</p>}
    </CardContent>
  </Card>;
}

function displayBalance(status: CngnWalletStatus | null) {
  if (!status?.balance_base_units || status.decimals === null) return "Unverified";
  const decimals = status.decimals;
  const padded = status.balance_base_units.padStart(decimals + 1, "0");
  const whole = decimals === 0 ? padded : padded.slice(0, -decimals);
  const fraction = decimals === 0 ? "" : padded.slice(-decimals).replace(/0+$/, "");
  return `${whole}${fraction ? `.${fraction}` : ""} cNGN`;
}

function Detail({ label, value }: { label: string; value: string }) { return <div><p className="text-xs text-muted-foreground">{label}</p><p className="mt-1 font-medium">{value}</p></div>; }

function Address({ label, value, hint }: { label: string; value?: string | null; hint?: string }) {
  const [copied, setCopied] = useState(false);
  async function copy() { if (!value) return; await navigator.clipboard.writeText(value); setCopied(true); window.setTimeout(() => setCopied(false), 1500); }
  return <div className="sm:col-span-2"><p className="text-xs text-muted-foreground">{label}</p><div className="mt-1 flex items-start gap-2"><p className="min-w-0 flex-1 break-all font-mono text-xs">{value ?? "Not available"}</p>{value && <Button size="icon" variant="ghost" className="-mt-1 size-7 shrink-0" onClick={() => void copy()} aria-label={`Copy ${label.toLowerCase()}`}>{copied ? <Check className="text-success" /> : <Copy />}</Button>}</div>{hint && <p className="mt-1 text-xs text-muted-foreground">{hint}</p>}</div>;
}
