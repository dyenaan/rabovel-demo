"use client";

import Link from "next/link";
import { Repeat } from "lucide-react";

import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { PriceChange } from "@/components/financial/price-change";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatCompactMoney, formatPrice } from "@/lib/formatters";
import type { Market } from "@/types";
import { useMarkets } from "../hooks/use-markets";

const columns: DataTableColumn<Market>[] = [
  {
    accessorKey: "symbol",
    header: "Market",
    cell: ({ row }) => (
      <Link href={`/markets/${row.original.marketId}`} className="hover:underline">
        <p className="font-medium text-foreground">{row.original.symbol}</p>
        <p className="text-xs text-muted-foreground">{row.original.assetName}</p>
      </Link>
    ),
  },
  {
    accessorKey: "lastPrice",
    header: "Last Price",
    cell: ({ row }) => <span className="font-tabular">{formatPrice(row.original.lastPrice)}</span>,
  },
  {
    id: "change",
    header: "24h Change",
    cell: ({ row }) =>
      row.original.priceChangePercent24h ? (
        <PriceChange
          absoluteValue={row.original.priceChange24h}
          percentValue={row.original.priceChangePercent24h}
        />
      ) : (
        <span className="text-muted-foreground">—</span>
      ),
  },
  {
    accessorKey: "volume24h",
    header: "24h Volume",
    cell: ({ row }) => (
      <span className="font-tabular">
        {row.original.volume24h ? formatCompactMoney(row.original.volume24h) : "—"}
      </span>
    ),
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <StatusBadge status={row.original.status} />,
  },
  {
    id: "actions",
    header: "",
    cell: ({ row }) => (
      <Link
        href={`/trade?market=${row.original.marketId}`}
        className="text-sm font-medium text-primary hover:underline"
      >
        Trade
      </Link>
    ),
  },
];

export function MarketsTable() {
  const { data, isPending } = useMarkets();

  return (
    <DataTable
      columns={columns}
      data={data ?? []}
      isLoading={isPending}
      emptyState={<EmptyState icon={Repeat} title="No markets available" />}
    />
  );
}
