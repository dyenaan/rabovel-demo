"use client";

import { Skeleton } from "@/components/ui/skeleton";
import { useOrderBook } from "@/features/markets/hooks/use-markets";
import { OrderBookSide } from "./order-book-side";
import { SpreadIndicator } from "./spread-indicator";

export function OrderBook({ marketId }: { marketId: string }) {
  const { data, isPending } = useOrderBook(marketId);

  if (isPending || !data) {
    return <Skeleton className="h-96 w-full" />;
  }

  return (
    <div>
      <div className="grid grid-cols-3 px-3 pb-1 text-[10px] font-medium tracking-wide text-muted-foreground uppercase">
        <span>Price</span>
        <span className="text-right">Size</span>
        <span className="text-right">Total</span>
      </div>
      <OrderBookSide levels={data.asks} side="ask" />
      <SpreadIndicator bestBid={data.bids[0]?.price} bestAsk={data.asks.at(-1)?.price} />
      <OrderBookSide levels={data.bids} side="bid" />
    </div>
  );
}
