"use client";

import Link from "next/link";
import { Repeat } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/shared/empty-state";
import { ErrorState } from "@/components/shared/error-state";
import { LoadingState } from "@/components/shared/loading-state";
import { PriceChange } from "@/components/financial/price-change";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatCompactMoney, formatPrice } from "@/lib/formatters";
import { PriceChart } from "@/features/trading/components/price-chart";
import { RecentTrades } from "@/features/trading/components/recent-trades";
import { useMarket } from "../hooks/use-markets";

export function MarketDetailView({ marketId }: { marketId: string }) {
  const { data: market, isPending, isError, refetch } = useMarket(marketId);

  if (isPending) return <LoadingState label="Loading market…" />;
  if (isError) return <ErrorState onRetry={() => refetch()} />;
  if (!market) return <EmptyState icon={Repeat} title="Market not found" />;

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 border-b pb-6 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-semibold text-foreground">{market.symbol}</h1>
            <StatusBadge status={market.status} />
          </div>
          <p className="text-sm text-muted-foreground">{market.assetName}</p>
        </div>
        <div className="grid grid-cols-2 gap-6 sm:grid-cols-4">
          <div>
            <p className="text-[10px] text-muted-foreground uppercase">Last Price</p>
            <p className="font-tabular text-lg font-semibold">{formatPrice(market.lastPrice)}</p>
          </div>
          <div>
            <p className="text-[10px] text-muted-foreground uppercase">24h Change</p>
            {market.priceChangePercent24h ? (
              <PriceChange absoluteValue={market.priceChange24h} percentValue={market.priceChangePercent24h} />
            ) : (
              <span className="text-sm text-muted-foreground">—</span>
            )}
          </div>
          <div>
            <p className="text-[10px] text-muted-foreground uppercase">24h Volume</p>
            <p className="font-tabular text-sm">{market.volume24h ? formatCompactMoney(market.volume24h) : "—"}</p>
          </div>
          <Button asChild>
            <Link href={`/trade?market=${market.marketId}`}>Trade</Link>
          </Button>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[1fr_320px]">
        <Card>
          <CardContent className="pt-6">
            <PriceChart marketId={market.marketId} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Recent Trades</CardTitle>
          </CardHeader>
          <CardContent className="pt-2">
            <RecentTrades marketId={market.marketId} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
