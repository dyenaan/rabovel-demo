"use client";

import { useState } from "react";
import Link from "next/link";
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
  return <PageContainer><PageHeader title="Live assets" description="Explore demo investments, review prices, and manage your holdings." actions={<Button variant="outline" onClick={() => void refetch()}><RefreshCw />Refresh balances</Button>} /><div className="space-y-6">
    {isError ? <ErrorState onRetry={() => refetch()} /> : isPending ? <div className="grid gap-5 sm:grid-cols-2"><Skeleton className="h-44" /><Skeleton className="h-72" /></div> : <>
      <Card><CardHeader><div className="flex flex-wrap items-start justify-between gap-3"><div><CardTitle className="flex items-center gap-2"><WalletCards className="size-5" />Available to invest</CardTitle><CardDescription>Purchases are paid from your cNGN balance.</CardDescription></div><Badge variant={data?.cngn.ready ? "success" : "warning"}>{data?.cngn.ready ? "Payments set up" : "Set up payments"}</Badge></div></CardHeader><CardContent className="grid gap-3 text-sm sm:grid-cols-2"><Value label="Available balance" value={formatTokenBalance(data?.cngn.balance_base_units, data?.cngn.decimals)} /><Button asChild variant="outline"><Link href="/wallet">Manage payments</Link></Button><details className="sm:col-span-2"><summary className="cursor-pointer text-xs text-muted-foreground">Technical details</summary><div className="mt-3 grid gap-3 sm:grid-cols-2"><Value label="Wallet address" value={data?.wallet_address ?? "Unavailable"} mono /><Value label="Payment account" value={data?.cngn.token_account ?? "Not set up"} mono /><Value label="Network" value={data?.cngn.network ?? "Unavailable"} /></div></details></CardContent></Card>
      {!data?.assets.length ? <EmptyState icon={Building2} title="No investments available yet" description="Assets will appear here once the issuer has completed setup and made units available." /> : <div className="grid gap-5 lg:grid-cols-2">{data.assets.map((asset) => <LiveAssetCard key={asset.asset_id} asset={asset} paymentReady={Boolean(data.cngn.ready && data.cngn.balance_base_units !== "0")} paymentDecimals={data.cngn.decimals ?? 6} />)}</div>}
    </>}
  </div></PageContainer>;
}

function LiveAssetCard({ asset, paymentReady, paymentDecimals }: { asset: InvestorCatalogAsset; paymentReady: boolean; paymentDecimals: number }) {
  const [quoteOpen, setQuoteOpen] = useState(false);
  const ready = paymentReady && asset.issuer_inventory !== "0";
  return <><Card><CardHeader><div className="flex flex-wrap items-start justify-between gap-3"><div><Badge variant="secondary">{asset.asset_type}</Badge><CardTitle className="mt-3">{asset.name}</CardTitle><CardDescription>{asset.ticker} · Pay in {asset.settlement_currency}</CardDescription></div><Badge variant={ready ? "success" : "warning"}>{ready ? "Available to buy" : "Action required"}</Badge></div></CardHeader><CardContent className="space-y-4"><p className="text-sm text-muted-foreground">{asset.description}</p><div className="rounded-md border bg-primary/5 p-3"><div className="flex flex-wrap items-center justify-between gap-3"><div><p className="text-xs text-muted-foreground">Market price</p><p className="mt-1 text-2xl font-semibold tabular-nums">{formatTokenBalance(asset.price_per_unit, asset.price_decimals)}</p></div><Badge variant="secondary">Simulated feed</Badge></div><p className="mt-2 text-xs text-muted-foreground">Updated {new Date(asset.price_updated_at * 1000).toLocaleTimeString()}</p></div><div className="rounded-md border bg-muted/20 p-3 text-sm"><p className="flex items-center gap-2 font-medium"><ShieldCheck className="size-4 text-primary" />Verified backing <Badge variant="secondary">Simulated</Badge></p><p className="mt-2 text-muted-foreground">{asset.backing.summary}</p><p className="mt-2 text-xs text-muted-foreground">Evidence: {asset.backing.document_name}</p></div><div className="grid gap-3 text-sm sm:grid-cols-2"><Value label="Units available" value={`${Number(asset.issuer_inventory).toLocaleString()} units`} /><Value label="Your holdings" value={`${Number(asset.investor_balance).toLocaleString()} units`} /></div><details><summary className="cursor-pointer text-xs text-muted-foreground">Technical details</summary><div className="mt-3 grid gap-3 sm:grid-cols-2"><Value label="Network" value={asset.network} /><Value label="Asset mint address" value={asset.mint_address} mono /><Value label="Holdings account" value={asset.investor_token_account} mono /></div></details><div className="flex items-center gap-2 text-xs text-muted-foreground"><ShieldCheck className="size-4 text-primary" />{asset.disclosure}</div><Button className="w-full" disabled={!ready} onClick={() => setQuoteOpen(true)}>Buy {asset.ticker}</Button>{!asset.investor_account_ready && <p className="text-xs text-amber-700 dark:text-amber-300">Your holdings account is not active yet.</p>}{!paymentReady && <p className="text-xs text-amber-700 dark:text-amber-300">Add demo funds in Account & Payments before buying.</p>}</CardContent></Card><InvestorQuoteDialog asset={asset} open={quoteOpen} onOpenChange={setQuoteOpen} paymentDecimals={paymentDecimals} /></>;
}

function Value({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><p className="text-xs text-muted-foreground">{label}</p><p className={`mt-1 break-all ${mono ? "font-mono text-xs" : "font-medium"}`}>{value}</p></div>; }

function formatTokenBalance(value?: string | null, decimals?: number | null) {
  if (value === null || value === undefined || decimals === null || decimals === undefined) return "Unavailable";
  const padded = value.padStart(decimals + 1, "0"); const whole = decimals ? padded.slice(0, -decimals) : padded; const fraction = decimals ? padded.slice(-decimals).replace(/0+$/, "") : "";
  return `${whole}${fraction ? `.${fraction}` : ""} cNGN`;
}
