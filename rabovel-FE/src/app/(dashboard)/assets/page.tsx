"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { Building2, RefreshCw, Search, ShieldCheck, WalletCards } from "lucide-react";
import { PageContainer } from "@/components/layout/page-container";
import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { ErrorState } from "@/components/shared/error-state";
import { PageHeader } from "@/components/shared/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { InvestorQuoteDialog } from "@/features/assets/components/investor-quote-dialog";
import { useAssets, useInvestorCatalog } from "@/features/assets/hooks/use-assets";
import { formatPrice } from "@/lib/formatters";
import type { Asset, InvestorCatalogAsset } from "@/types";

/** Assets are the instrument catalog: NGX-listed and issuer-minted alike. */
export default function AssetsMarketplacePage() {
  const assets = useAssets();
  // Needs a linked wallet; the catalog below renders without it.
  const catalog = useInvestorCatalog();
  const [query, setQuery] = useState("");
  const rows = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return (assets.data ?? []).filter((asset) => !needle || asset.symbol.toLowerCase().includes(needle) || asset.name.toLowerCase().includes(needle));
  }, [assets.data, query]);
  const onChain = catalog.data?.assets ?? [];

  const columns: DataTableColumn<Asset>[] = [
    { accessorKey: "symbol", header: "Asset", cell: ({ row }) => <Link href={`/assets/${encodeURIComponent(row.original.assetId)}`} className="hover:underline"><p className="font-medium text-foreground">{row.original.symbol}</p><p className="text-xs text-muted-foreground">{row.original.name}</p></Link> },
    { id: "source", header: "Source", cell: ({ row }) => row.original.origin === "issuer" ? <Badge variant="outline">Issuer listing</Badge> : <Badge variant="secondary">NGX-listed</Badge> },
    { accessorKey: "marketPrice", header: "Price (NGN)", cell: ({ row }) => <span className="font-tabular">{row.original.marketPrice ? formatPrice(row.original.marketPrice) : "—"}</span> },
    { id: "actions", header: "", cell: ({ row }) => row.original.marketId ? <Link href={`/trade?market=${row.original.marketId}`} className="px-2 text-sm font-medium text-primary hover:underline">Trade</Link> : null },
  ];

  return <PageContainer><PageHeader title="Assets" description="Explore NGX-listed instruments and issuer investments. Trading and prices are simulated." actions={<Button variant="outline" onClick={() => { void assets.refetch(); void catalog.refetch(); }}><RefreshCw />Refresh</Button>} /><div className="space-y-6">
    {catalog.data && <Card><CardHeader><div className="flex flex-wrap items-start justify-between gap-3"><div><CardTitle className="flex items-center gap-2"><WalletCards className="size-5" />Available to invest</CardTitle><CardDescription>Purchases are paid from your cNGN balance.</CardDescription></div><Badge variant={catalog.data?.cngn.ready ? "success" : "warning"}>{catalog.data?.cngn.ready ? "Payments set up" : "Set up payments"}</Badge></div></CardHeader><CardContent className="grid gap-3 text-sm sm:grid-cols-2"><Value label="Available balance" value={formatTokenBalance(catalog.data?.cngn.balance_base_units, catalog.data?.cngn.decimals)} /><Button asChild variant="outline"><Link href="/wallet">Manage payments</Link></Button><details className="sm:col-span-2"><summary className="cursor-pointer text-xs text-muted-foreground">Technical details</summary><div className="mt-3 grid gap-3 sm:grid-cols-2"><Value label="Wallet address" value={catalog.data?.wallet_address ?? "Unavailable"} mono /><Value label="Payment account" value={catalog.data?.cngn.token_account ?? "Not set up"} mono /><Value label="Network" value={catalog.data?.cngn.network ?? "Unavailable"} /></div></details></CardContent></Card>}
    {catalog.isError && <Card><CardContent className="space-y-3 pt-6"><p className="text-sm text-muted-foreground">Your payment balance and issuer purchases are unavailable. Connect Phantom in Account & Payments if you have not already. You can still browse the catalog below.</p><div className="flex flex-wrap gap-2"><Button asChild variant="outline"><Link href="/wallet">Manage payments</Link></Button><Button variant="outline" onClick={() => void catalog.refetch()}>Retry balance</Button></div></CardContent></Card>}
    {onChain.length > 0 && <div className="space-y-3"><h2 className="text-lg font-semibold">Buy issuer assets</h2><div className="grid gap-5 lg:grid-cols-2">{onChain.map((asset) => <LiveAssetCard key={asset.asset_id} asset={asset} paymentReady={Boolean(catalog.data?.cngn.ready && catalog.data.cngn.balance_base_units !== "0")} paymentDecimals={catalog.data?.cngn.decimals ?? 6} />)}</div></div>}
    {assets.isError ? <ErrorState onRetry={() => assets.refetch()} /> : <Card><CardHeader><div className="flex flex-wrap items-center justify-between gap-3"><div><CardTitle>All assets</CardTitle><CardDescription>{assets.data ? `${assets.data.length} investments to explore` : "Loading catalog…"}</CardDescription></div><div className="relative w-full sm:w-64"><Search className="absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" /><Input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search ticker or name" className="pl-8" aria-label="Search assets" /></div></div></CardHeader><CardContent><DataTable columns={columns} data={rows} isLoading={assets.isPending} emptyState={<EmptyState icon={Building2} title="No matching assets" description="Try another ticker or name." />} /></CardContent></Card>}
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
