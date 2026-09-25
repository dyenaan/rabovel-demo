"use client";

import { useState } from "react";
import { Building2, RefreshCw, ShieldCheck, WalletCards } from "lucide-react";
import { PageContainer } from "@/components/layout/page-container";
import { EmptyState } from "@/components/shared/empty-state";
import { ErrorState } from "@/components/shared/error-state";
import { PageHeader } from "@/components/shared/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { InvestorQuoteDialog } from "@/features/assets/components/investor-quote-dialog";
import { useInvestorCatalog } from "@/features/assets/hooks/use-assets";
import type { InvestorCatalogAsset } from "@/types";

export default function AssetsMarketplacePage() {
  const { data, isPending, isError, refetch } = useInvestorCatalog();
  return <PageContainer><PageHeader title="Live assets" description="Chain-verified simulated assets with your current wallet balances." actions={<Button variant="outline" onClick={() => void refetch()}><RefreshCw />Refresh balances</Button>} /><div className="space-y-6">
    {isError ? <ErrorState onRetry={() => refetch()} /> : isPending ? <div className="grid gap-5 sm:grid-cols-2"><Skeleton className="h-44" /><Skeleton className="h-72" /></div> : <>
      <Card><CardHeader><div className="flex items-start justify-between gap-3"><div><CardTitle className="flex items-center gap-2"><WalletCards className="size-5" />Purchase readiness</CardTitle><CardDescription>Live payment balance for your linked Phantom wallet.</CardDescription></div><Badge variant={data?.cngn.ready ? "success" : "warning"}>{data?.cngn.ready ? "cNGN ready" : "cNGN not ready"}</Badge></div></CardHeader><CardContent className="grid gap-3 text-sm sm:grid-cols-2"><Value label="Wallet" value={data?.wallet_address ?? "Unavailable"} mono /><Value label="cNGN balance" value={formatTokenBalance(data?.cngn.balance_base_units, data?.cngn.decimals)} /><Value label="cNGN token account" value={data?.cngn.token_account ?? "Create and fund this account from Wallet & Custody"} mono /><Value label="Network" value={data?.cngn.network ?? "Unavailable"} /></CardContent></Card>
      {!data?.assets.length ? <EmptyState icon={Building2} title="No issued assets are ready" description="An asset appears only after mint setup and initial inventory issuance are confirmed on-chain." /> : <div className="grid gap-5 lg:grid-cols-2">{data.assets.map((asset) => <LiveAssetCard key={asset.asset_id} asset={asset} paymentReady={Boolean(data.cngn.ready && data.cngn.balance_base_units !== "0")} paymentDecimals={data.cngn.decimals ?? 6} />)}</div>}
    </>}
  </div></PageContainer>;
}

function LiveAssetCard({ asset, paymentReady, paymentDecimals }: { asset: InvestorCatalogAsset; paymentReady: boolean; paymentDecimals: number }) {
  const [quoteOpen, setQuoteOpen] = useState(false);
  const ready = paymentReady && asset.issuer_inventory !== "0";
  return <><Card><CardHeader><div className="flex items-start justify-between gap-3"><div><Badge variant="secondary">{asset.asset_type}</Badge><CardTitle className="mt-3">{asset.name}</CardTitle><CardDescription>{asset.ticker} · {asset.network} · settles in {asset.settlement_currency}</CardDescription></div><Badge variant={ready ? "success" : "warning"}>{ready ? "Ready for quote" : "Action required"}</Badge></div></CardHeader><CardContent className="space-y-4"><p className="text-sm text-muted-foreground">{asset.description}</p><div className="rounded-md border bg-muted/20 p-3 text-sm"><p className="flex items-center gap-2 font-medium"><ShieldCheck className="size-4 text-primary" />Verified backing <Badge variant="secondary">Simulated</Badge></p><p className="mt-2 text-muted-foreground">{asset.backing.summary}</p><p className="mt-2 text-xs text-muted-foreground">Evidence: {asset.backing.document_name}</p></div><div className="grid gap-3 text-sm sm:grid-cols-2"><Value label="Issuer inventory" value={`${Number(asset.issuer_inventory).toLocaleString()} units`} /><Value label="Your equity balance" value={`${Number(asset.investor_balance).toLocaleString()} units`} /><Value label="Equity mint" value={asset.mint_address} mono /><Value label="Your equity token account" value={asset.investor_token_account} mono /></div><div className="flex items-center gap-2 text-xs text-muted-foreground"><ShieldCheck className="size-4 text-primary" />{asset.disclosure}</div><Button className="w-full" disabled={!ready} onClick={() => setQuoteOpen(true)}>Buy {asset.ticker}</Button>{!asset.investor_account_ready && <p className="text-xs text-amber-700 dark:text-amber-300">Your equity token account is not active yet.</p>}{!paymentReady && <p className="text-xs text-amber-700 dark:text-amber-300">Create and fund your cNGN account before requesting a quote.</p>}</CardContent></Card><InvestorQuoteDialog asset={asset} open={quoteOpen} onOpenChange={setQuoteOpen} paymentDecimals={paymentDecimals} /></>;
}

function Value({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><p className="text-xs text-muted-foreground">{label}</p><p className={`mt-1 break-all ${mono ? "font-mono text-xs" : "font-medium"}`}>{value}</p></div>; }

function formatTokenBalance(value?: string | null, decimals?: number | null) {
  if (value === null || value === undefined || decimals === null || decimals === undefined) return "Unavailable";
  const padded = value.padStart(decimals + 1, "0"); const whole = decimals ? padded.slice(0, -decimals) : padded; const fraction = decimals ? padded.slice(-decimals).replace(/0+$/, "") : "";
  return `${whole}${fraction ? `.${fraction}` : ""} cNGN`;
}
