"use client";

import { Banknote } from "lucide-react";

import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { Money } from "@/components/financial/money";
import { Percentage } from "@/components/financial/percentage";
import { StatusBadge } from "@/components/shared/status-badge";
import { mockAssets } from "@/mocks/assets.mock";
import { mockReserveStatus } from "@/mocks/primary-market.mock";
import type { ReserveStatus } from "@/features/primary-market/types/primary-market.types";

const columns: DataTableColumn<ReserveStatus>[] = [
  {
    accessorKey: "assetId",
    header: "Asset",
    cell: ({ row }) => {
      const asset = mockAssets.find((a) => a.assetId === row.original.assetId);
      return <span className="font-medium">{asset?.name ?? row.original.assetId}</span>;
    },
  },
  {
    accessorKey: "totalIssued",
    header: "Issued",
    cell: ({ row }) => <Money value={row.original.totalIssued} compact />,
  },
  {
    accessorKey: "totalReserved",
    header: "Reserved",
    cell: ({ row }) => <Money value={row.original.totalReserved} compact />,
  },
  {
    accessorKey: "coveragePercent",
    header: "Coverage",
    cell: ({ row }) => <Percentage value={row.original.coveragePercent} signed={false} />,
  },
  {
    id: "status",
    header: "Status",
    cell: ({ row }) => (
      <StatusBadge status={Number(row.original.coveragePercent) >= 100 ? "RECONCILED" : "UNDER_REVIEW"} />
    ),
  },
];

export function ReserveAssetsTable() {
  return (
    <DataTable
      columns={columns}
      data={mockReserveStatus}
      emptyState={<EmptyState icon={Banknote} title="No reserve data" />}
    />
  );
}
