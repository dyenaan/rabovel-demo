"use client";

import Link from "next/link";
import { ShieldCheck } from "lucide-react";

import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { Money } from "@/components/financial/money";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatDateTime } from "@/lib/formatters";
import type { Settlement } from "@/types";
import { useSettlements } from "../hooks/use-settlements";

const columns: DataTableColumn<Settlement>[] = [
  {
    accessorKey: "settlementId",
    header: "Settlement",
    cell: ({ row }) => (
      <Link href={`/settlements/${row.original.settlementId}`} className="font-medium hover:underline">
        {row.original.settlementId}
      </Link>
    ),
  },
  {
    id: "assetLeg",
    header: "Asset Leg",
    cell: ({ row }) => (
      <span className="font-tabular">
        {row.original.assetLeg.amount} {row.original.assetLeg.assetOrCurrency}
      </span>
    ),
  },
  {
    id: "cashLeg",
    header: "Cash Leg",
    cell: ({ row }) => <Money value={row.original.cashLeg.amount} currency={row.original.cashLeg.assetOrCurrency} />,
  },
  {
    accessorKey: "custodyRoute",
    header: "Custody Route",
    cell: ({ row }) => row.original.custodyRoute ?? "—",
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <StatusBadge status={row.original.status} />,
  },
  {
    accessorKey: "updatedAt",
    header: "Updated",
    cell: ({ row }) => (
      <span className="text-xs text-muted-foreground">{formatDateTime(row.original.updatedAt)}</span>
    ),
  },
];

export function SettlementsTable() {
  const { data, isPending } = useSettlements();

  return (
    <DataTable
      columns={columns}
      data={data ?? []}
      isLoading={isPending}
      emptyState={<EmptyState icon={ShieldCheck} title="No settlements yet" />}
    />
  );
}
