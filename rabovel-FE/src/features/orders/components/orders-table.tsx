"use client";

import Link from "next/link";
import { ListOrdered } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ConfirmDialog } from "@/components/shared/confirm-dialog";
import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { createIdempotencyKey } from "@/lib/api/client";
import { formatDateTime, formatPrice, formatQuantity } from "@/lib/formatters";
import { OPEN_ORDER_STATUSES, type Order } from "@/types";
import { useCancelOrder } from "../hooks/use-place-order";

function buildColumns(onCancel: (order: Order) => void): DataTableColumn<Order>[] {
  return [
    {
      accessorKey: "marketSymbol",
      header: "Market",
      cell: ({ row }) => (
        <Link href={`/orders/${row.original.orderId}`} className="font-medium hover:underline">
          {row.original.marketSymbol}
        </Link>
      ),
    },
    {
      accessorKey: "side",
      header: "Side",
      cell: ({ row }) => (
        <Badge variant={row.original.side === "BUY" ? "success" : "destructive"}>
          {row.original.side}
        </Badge>
      ),
    },
    { accessorKey: "orderType", header: "Type" },
    {
      accessorKey: "limitPrice",
      header: "Price",
      cell: ({ row }) => (
        <span className="font-tabular">
          {row.original.limitPrice ? formatPrice(row.original.limitPrice) : "Market"}
        </span>
      ),
    },
    {
      accessorKey: "quantity",
      header: "Qty (Exec/Total)",
      cell: ({ row }) => (
        <span className="font-tabular">
          {formatQuantity(row.original.executedQuantity)} / {formatQuantity(row.original.quantity)}
        </span>
      ),
    },
    {
      accessorKey: "status",
      header: "Status",
      cell: ({ row }) => <StatusBadge status={row.original.status} />,
    },
    {
      accessorKey: "createdAt",
      header: "Created",
      cell: ({ row }) => (
        <span className="text-xs text-muted-foreground">{formatDateTime(row.original.createdAt)}</span>
      ),
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) =>
        OPEN_ORDER_STATUSES.includes(row.original.status) ? (
          <ConfirmDialog
            trigger={
              <Button variant="ghost" size="sm">
                Cancel
              </Button>
            }
            title="Cancel this order?"
            description={`This will cancel your ${row.original.side} order for ${row.original.marketSymbol}.`}
            confirmLabel="Cancel order"
            destructive
            onConfirm={() => onCancel(row.original)}
          />
        ) : null,
    },
  ];
}

export function OrdersTable({ orders, isLoading }: { orders: Order[]; isLoading?: boolean }) {
  const cancelOrder = useCancelOrder();

  const columns = buildColumns((order) =>
    cancelOrder.mutate({ orderId: order.orderId, idempotencyKey: createIdempotencyKey() }),
  );

  return (
    <DataTable
      columns={columns}
      data={orders}
      isLoading={isLoading}
      emptyState={<EmptyState icon={ListOrdered} title="No orders" description="Orders you place will appear here." />}
    />
  );
}
