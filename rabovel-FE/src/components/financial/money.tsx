import { formatCompactMoney, formatMoney } from "@/lib/formatters";
import { cn } from "@/lib/utils";

export function Money({
  value,
  currency = "NGN",
  compact = false,
  className,
}: {
  value: string | number | undefined | null;
  currency?: string;
  compact?: boolean;
  className?: string;
}) {
  const formatted = compact
    ? formatCompactMoney(value, currency)
    : formatMoney(value, { currency });

  return <span className={cn("font-tabular", className)}>{formatted}</span>;
}
