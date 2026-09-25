"use client";

import Link from "next/link";
import { Repeat } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatDateTime, formatPrice, formatQuantity } from "@/lib/formatters";
import type { Trade } from "@/types";
import { useTrades } from "../hooks/use-trades";

const columns: DataTableColumn<Trade>[] = [
  {
    accessorKey: "marketSymbol",
    header: "Market",
    cell: ({ row }) => <span className="font-medium">{row.original.marketSymbol}</span>,
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
  {
    accessorKey: "executionPrice",
    header: "Execution Price",
    cell: ({ row }) => <span className="font-tabular">{formatPrice(row.original.executionPrice)}</span>,
  },
  {
    accessorKey: "quantity",
    header: "Quantity",
    cell: ({ row }) => <span className="font-tabular">{formatQuantity(row.original.quantity)}</span>,
  },
  {
    accessorKey: "executedAt",
    header: "Executed",
    cell: ({ row }) => (
      <span className="text-xs text-muted-foreground">{formatDateTime(row.original.executedAt)}</span>
    ),
  },
  {
    id: "settlement",
    header: "Settlement",
    cell: ({ row }) =>
      row.original.settlementId ? (
        <Link href={`/settlements/${row.original.settlementId}`}>
          <StatusBadge status={row.original.settlementStatus} />
        </Link>
      ) : (
        <StatusBadge status={row.original.settlementStatus} />
      ),
  },
];

export function TradesTable() {
  const { data, isPending } = useTrades();

  return (
    <DataTable
      columns={columns}
      data={data ?? []}
      isLoading={isPending}
      emptyState={<EmptyState icon={Repeat} title="No trades yet" />}
    />
  );
}
