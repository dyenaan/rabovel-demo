"use client";

import Link from "next/link";
import { Repeat, Star } from "lucide-react";

import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { PriceChange } from "@/components/financial/price-change";
import { StatusBadge } from "@/components/shared/status-badge";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { formatPrice } from "@/lib/formatters";
import { useMarketWatchlistStore } from "@/stores/market-watchlist-store";
import type { Market } from "@/types";
import { useMarkets } from "../hooks/use-markets";

export function MarketsTable({ watchlistOnly = false }: { watchlistOnly?: boolean }) {
  const { data, isPending } = useMarkets();
  const marketIds = useMarketWatchlistStore((state) => state.marketIds);
  const toggleMarket = useMarketWatchlistStore((state) => state.toggleMarket);
  const markets = watchlistOnly
    ? (data ?? []).filter((market) => marketIds.includes(market.marketId))
    : (data ?? []);

  const columns: DataTableColumn<Market>[] = [
    {
      accessorKey: "symbol",
      header: "Market",
      cell: ({ row }) => (
        <Link href={`/markets/${row.original.marketId}`} className="hover:underline">
          <p className="font-medium text-foreground">
            {row.original.baseAsset}
            <sup
              aria-label="simulated tokenized share"
              title="Simulated tokenized share"
              className="ml-0.5 align-super text-[9px] font-semibold text-primary"
            >
              r
            </sup>
            /{row.original.quoteAsset}
          </p>
          <p className="text-xs text-muted-foreground">{row.original.assetName}</p>
        </Link>
      ),
    },
    {
      accessorKey: "lastPrice",
      header: "Last Price",
      cell: ({ row }) => <span className="font-tabular">{formatPrice(row.original.lastPrice)}</span>,
    },
    {
      id: "change",
      header: "24h Change",
      cell: ({ row }) =>
        row.original.priceChangePercent24h ? (
          <PriceChange
            absoluteValue={row.original.priceChange24h}
            percentValue={row.original.priceChangePercent24h}
          />
        ) : (
          <span className="text-muted-foreground">—</span>
        ),
    },
    {
      accessorKey: "volume24h",
      header: "24h Volume (units)",
      cell: ({ row }) => (
        <span className="font-tabular">{formatCompactVolume(row.original.volume24h)}</span>
      ),
    },
    {
      accessorKey: "status",
      header: watchlistOnly ? "Demo token" : "Status",
      cell: ({ row }) =>
        watchlistOnly ? (
          <div className="flex flex-wrap items-center gap-1.5">
            <Badge variant="success">Minted</Badge>
            <Badge variant="outline">Tradable</Badge>
            {row.original.origin === "issuer" ? <Badge variant="outline">Issuer listing</Badge> : null}
          </div>
        ) : (
          <StatusBadge status={row.original.status} />
        ),
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => {
        const isWatched = marketIds.includes(row.original.marketId);
        return (
          <div className="flex items-center justify-end gap-1">
            <Button
              type="button"
              variant="ghost"
              size="icon"
              aria-label={
                isWatched
                  ? `Remove ${row.original.symbol} from watchlist`
                  : `Add ${row.original.symbol} to watchlist`
              }
              title={isWatched ? "Remove from watchlist" : "Add to watchlist"}
              onClick={() => toggleMarket(row.original.marketId)}
            >
              <Star className={isWatched ? "fill-current text-primary" : ""} />
            </Button>
            <Link
              href={`/trade?market=${row.original.marketId}`}
              className="px-2 text-sm font-medium text-primary hover:underline"
            >
              Trade
            </Link>
          </div>
        );
      },
    },
  ];

  return (
    <DataTable
      columns={columns}
      data={markets}
      isLoading={isPending}
      emptyState={
        watchlistOnly ? (
          <EmptyState
            icon={Star}
            title="Your watchlist is empty"
            description="Star markets to keep them close and follow their demo prices here."
            action={
              <Button asChild variant="outline">
                <Link href="/markets">Browse markets</Link>
              </Button>
            }
          />
        ) : (
          <EmptyState icon={Repeat} title="No markets available" />
        )
      }
    />
  );
}

function formatCompactVolume(value?: string) {
  if (!value) return "—";
  const volume = Number(value);
  if (!Number.isFinite(volume)) return "—";
  return new Intl.NumberFormat("en-NG", {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(volume);
}
