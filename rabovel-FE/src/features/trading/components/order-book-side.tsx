import type { OrderBookLevel } from "@/types";
import { OrderBookRow } from "./order-book-row";

export function OrderBookSide({
  levels,
  side,
}: {
  levels: OrderBookLevel[];
  side: "bid" | "ask";
}) {
  const maxTotal = Math.max(...levels.map((l) => Number(l.total)), 1);

  return (
    <div>
      {levels.map((level) => (
        <OrderBookRow
          key={level.price}
          level={level}
          side={side}
          depthPercent={(Number(level.total) / maxTotal) * 100}
        />
      ))}
    </div>
  );
}
