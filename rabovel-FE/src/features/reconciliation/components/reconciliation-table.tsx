"use client";

import { GitCompareArrows } from "lucide-react";

import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { Money } from "@/components/financial/money";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatDateTime } from "@/lib/formatters";
import { mockReconciliationRecords } from "@/mocks/compliance.mock";
import type { ReconciliationRecord } from "@/types";
import { MismatchDetails } from "./mismatch-details";

const columns: DataTableColumn<ReconciliationRecord>[] = [
  { accessorKey: "assetName", header: "Asset" },
  {
    accessorKey: "ledgerBalance",
    header: "Ledger",
    cell: ({ row }) => <Money value={row.original.ledgerBalance} compact />,
  },
  {
    accessorKey: "custodyBalance",
    header: "Custody",
    cell: ({ row }) => <Money value={row.original.custodyBalance} compact />,
  },
  {
    accessorKey: "onChainBalance",
    header: "On-Chain",
    cell: ({ row }) => <Money value={row.original.onChainBalance} compact />,
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <StatusBadge status={row.original.status} />,
  },
  {
    accessorKey: "lastCheckedAt",
    header: "Last Checked",
    cell: ({ row }) => (
      <span className="text-xs text-muted-foreground">{formatDateTime(row.original.lastCheckedAt)}</span>
    ),
  },
  {
    id: "actions",
    header: "",
    cell: ({ row }) => <MismatchDetails record={row.original} />,
  },
];

export function ReconciliationTable() {
  return (
    <DataTable
      columns={columns}
      data={mockReconciliationRecords}
      emptyState={<EmptyState icon={GitCompareArrows} title="No reconciliation records" />}
    />
  );
}
