import { Decimal, formatPrice } from "@/lib/formatters";

export function SpreadIndicator({ bestBid, bestAsk }: { bestBid?: string; bestAsk?: string }) {
  if (!bestBid || !bestAsk) return null;

  const spread = new Decimal(bestAsk).minus(bestBid);
  const spreadPercent = spread.div(bestAsk).times(100);

  return (
    <div className="flex items-center justify-between border-y bg-muted/40 px-3 py-1.5 text-xs">
      <span className="text-muted-foreground">Spread</span>
      <span className="font-tabular font-medium text-foreground">
        {formatPrice(spread.toString())} ({spreadPercent.toFixed(2)}%)
      </span>
    </div>
  );
}
