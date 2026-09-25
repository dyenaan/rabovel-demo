import { ArrowDown, ArrowUp, Minus } from "lucide-react";

import { formatMoney, formatPercentage, isPositive } from "@/lib/formatters";
import { cn } from "@/lib/utils";

export function PriceChange({
  absoluteValue,
  percentValue,
  className,
}: {
  absoluteValue?: string | number | null;
  percentValue: string | number | undefined | null;
  className?: string;
}) {
  const positive = isPositive(percentValue);
  const isZero = Number(percentValue ?? 0) === 0;
  const Icon = isZero ? Minus : positive ? ArrowUp : ArrowDown;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 font-tabular text-sm",
        isZero ? "text-muted-foreground" : positive ? "text-success" : "text-danger",
        className,
      )}
    >
      <Icon className="size-3.5" aria-hidden="true" />
      {absoluteValue !== undefined && absoluteValue !== null && (
        <span>{formatMoney(absoluteValue, { maximumFractionDigits: 2 })}</span>
      )}
      <span>({formatPercentage(percentValue, { signed: false })})</span>
    </span>
  );
}
