import { formatPrice, formatQuantity } from "@/lib/formatters";
import { cn } from "@/lib/utils";
import type { OrderBookLevel } from "@/types";

export function OrderBookRow({
  level,
  side,
  depthPercent,
}: {
  level: OrderBookLevel;
  side: "bid" | "ask";
  depthPercent: number;
}) {
  return (
    <div className="relative grid grid-cols-3 px-3 py-0.5 text-xs font-tabular">
      <div
        className={cn(
          "absolute inset-y-0 opacity-[0.08]",
          side === "bid" ? "right-0 bg-success" : "right-0 bg-danger",
        )}
        style={{ width: `${depthPercent}%` }}
        aria-hidden="true"
      />
      <span className={cn("relative", side === "bid" ? "text-success" : "text-danger")}>
        {formatPrice(level.price)}
      </span>
      <span className="relative text-right text-muted-foreground">{formatQuantity(level.size, 2)}</span>
      <span className="relative text-right text-muted-foreground">{formatQuantity(level.total, 2)}</span>
    </div>
  );
}
