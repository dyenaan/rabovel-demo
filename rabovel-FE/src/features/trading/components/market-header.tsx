"use client";

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { PriceChange } from "@/components/financial/price-change";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatCompactMoney, formatPrice } from "@/lib/formatters";
import { useMarkets } from "@/features/markets/hooks/use-markets";
import type { Market } from "@/types";

export function MarketHeader({
  market,
  onMarketChange,
}: {
  market?: Market;
  onMarketChange: (marketId: string) => void;
}) {
  const { data: markets } = useMarkets();

  if (!market) {
    return <Skeleton className="h-20 w-full" />;
  }

  return (
    <div className="flex flex-col gap-4 border-b pb-4 sm:flex-row sm:items-center sm:justify-between">
      <div className="flex items-center gap-4">
        <Select value={market.marketId} onValueChange={onMarketChange}>
          <SelectTrigger className="w-44">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {(markets ?? []).map((m) => (
              <SelectItem key={m.marketId} value={m.marketId}>
                {m.symbol}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <div>
          <p className="text-xs text-muted-foreground">{market.assetName}</p>
          <StatusBadge status={market.status} />
        </div>
      </div>

      <div className="grid grid-cols-2 gap-6 sm:grid-cols-4">
        <div>
          <p className="text-[10px] text-muted-foreground uppercase">Last Price</p>
          <p className="font-tabular text-lg font-semibold text-foreground">
            {formatPrice(market.lastPrice)}
          </p>
        </div>
        <div>
          <p className="text-[10px] text-muted-foreground uppercase">24h Change</p>
          {market.priceChangePercent24h ? (
            <PriceChange
              absoluteValue={market.priceChange24h}
              percentValue={market.priceChangePercent24h}
            />
          ) : (
            <span className="text-sm text-muted-foreground">—</span>
          )}
        </div>
        <div>
          <p className="text-[10px] text-muted-foreground uppercase">24h High / Low</p>
          <p className="font-tabular text-sm text-foreground">
            {market.high24h ? formatPrice(market.high24h) : "—"} / {market.low24h ? formatPrice(market.low24h) : "—"}
          </p>
        </div>
        <div>
          <p className="text-[10px] text-muted-foreground uppercase">24h Volume</p>
          <p className="font-tabular text-sm text-foreground">
            {market.volume24h ? formatCompactMoney(market.volume24h) : "—"}
          </p>
        </div>
      </div>
    </div>
  );
}
