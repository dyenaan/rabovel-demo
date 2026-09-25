"use client";

import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { fetchCngnPaymentAsset, type PaymentAssetStatus } from "@/features/auth/api/backend-auth";
import { useAuthStore } from "@/stores/auth-store";

export function PaymentAssetStatusCard() {
  const token = useAuthStore((state) => state.token);
  const [status, setStatus] = useState<PaymentAssetStatus | null>(null);

  useEffect(() => {
    if (!token) return;
    void fetchCngnPaymentAsset(token).then(setStatus).catch(() => undefined);
  }, [token]);

  return <Card className="md:col-span-2"><CardHeader><div className="flex flex-wrap items-start justify-between gap-3"><div><CardTitle>CNGN payment rail</CardTitle><CardDescription className="mt-1">Trusted Solana configuration and live RPC verification for settlement.</CardDescription></div><Badge variant={status?.ready ? "success" : "warning"}>{status?.ready ? "Ready" : "Not ready"}</Badge></div></CardHeader><CardContent className="grid gap-3 text-sm sm:grid-cols-2"><Detail label="Network" value={status?.network ?? "Loading…"} /><Detail label="Token program" value={status?.token_program ?? "Unverified"} /><Detail label="Decimals" value={status?.decimals?.toString() ?? "Unverified"} /><Detail label="Broker account" value={status?.broker_token_account ?? "Not configured"} /><div className="sm:col-span-2"><Detail label="Mint" value={status?.mint_address ?? "Not configured"} mono /></div>{status?.error && <p className="sm:col-span-2 text-amber-700 dark:text-amber-300">{status.error}</p>}</CardContent></Card>;
}

function Detail({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return <div><p className="text-xs text-muted-foreground">{label}</p><p className={`mt-1 break-all ${mono ? "font-mono text-xs" : "font-medium"}`}>{value}</p></div>;
}
