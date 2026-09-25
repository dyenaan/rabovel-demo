"use client";

import { Skeleton } from "@/components/ui/skeleton";
import { formatPrice, formatQuantity } from "@/lib/formatters";
import { cn } from "@/lib/utils";
import { useRecentTrades } from "@/features/markets/hooks/use-markets";

export function RecentTrades({ marketId }: { marketId: string }) {
  const { data, isPending } = useRecentTrades(marketId);

  if (isPending || !data) {
    return <Skeleton className="h-96 w-full" />;
  }

  return (
    <div>
      <div className="grid grid-cols-3 px-3 pb-1 text-[10px] font-medium tracking-wide text-muted-foreground uppercase">
        <span>Price</span>
        <span className="text-right">Size</span>
        <span className="text-right">Time</span>
      </div>
      <div className="max-h-96 overflow-y-auto">
        {data.map((trade) => (
          <div key={trade.tradeId} className="grid grid-cols-3 px-3 py-0.5 text-xs font-tabular">
            <span className={cn(trade.side === "BUY" ? "text-success" : "text-danger")}>
              {formatPrice(trade.price)}
            </span>
            <span className="text-right text-muted-foreground">{formatQuantity(trade.quantity, 2)}</span>
            <span className="text-right text-muted-foreground">
              {new Date(trade.executedAt).toLocaleTimeString("en-US", {
                hour: "2-digit",
                minute: "2-digit",
                second: "2-digit",
              })}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
