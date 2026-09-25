import { formatPercentage, isPositive } from "@/lib/formatters";
import { cn } from "@/lib/utils";

export function Percentage({
  value,
  signed = true,
  colorize = false,
  className,
}: {
  value: string | number | undefined | null;
  signed?: boolean;
  colorize?: boolean;
  className?: string;
}) {
  const positive = isPositive(value);

  return (
    <span
      className={cn(
        "font-tabular",
        colorize && (positive ? "text-success" : "text-danger"),
        className,
      )}
    >
      {formatPercentage(value, { signed })}
    </span>
  );
}
