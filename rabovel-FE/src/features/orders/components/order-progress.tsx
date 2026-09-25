import { Progress } from "@/components/ui/progress";
import { formatQuantity } from "@/lib/formatters";
import { Decimal } from "@/lib/formatters";
import type { Order } from "@/types";

/**
 * Visualizes the order invariant executedQuantity + remainingQuantity = quantity.
 * All three values are always shown explicitly — never collapsed into a single
 * "filled" percentage — so the underlying amounts stay auditable.
 */
export function OrderProgress({ order }: { order: Pick<Order, "quantity" | "executedQuantity" | "remainingQuantity"> }) {
  const total = new Decimal(order.quantity || 0);
  const executedPercent = total.isZero()
    ? 0
    : new Decimal(order.executedQuantity || 0).div(total).times(100).toNumber();

  return (
    <div className="space-y-2">
      <Progress value={executedPercent} />
      <div className="grid grid-cols-3 gap-2 text-xs">
        <div>
          <p className="text-muted-foreground">Original Quantity</p>
          <p className="font-medium font-tabular text-foreground">{formatQuantity(order.quantity)}</p>
        </div>
        <div>
          <p className="text-muted-foreground">Executed</p>
          <p className="font-medium font-tabular text-success">{formatQuantity(order.executedQuantity)}</p>
        </div>
        <div>
          <p className="text-muted-foreground">Remaining</p>
          <p className="font-medium font-tabular text-foreground">{formatQuantity(order.remainingQuantity)}</p>
        </div>
      </div>
    </div>
  );
}
