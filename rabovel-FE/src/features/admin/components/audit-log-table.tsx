"use client";

import { FileSearch } from "lucide-react";

import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { formatDateTime } from "@/lib/formatters";
import { mockAuditLogs, type AuditLogEntry } from "@/mocks/audit.mock";

const columns: DataTableColumn<AuditLogEntry>[] = [
  {
    accessorKey: "timestamp",
    header: "Timestamp",
    cell: ({ row }) => (
      <span className="text-xs text-muted-foreground">{formatDateTime(row.original.timestamp)}</span>
    ),
  },
  { accessorKey: "actor", header: "Actor" },
  {
    accessorKey: "action",
    header: "Action",
    cell: ({ row }) => <span className="font-mono text-xs">{row.original.action}</span>,
  },
  { accessorKey: "target", header: "Target" },
  {
    accessorKey: "ipAddress",
    header: "IP Address",
    cell: ({ row }) => <span className="font-mono text-xs text-muted-foreground">{row.original.ipAddress}</span>,
  },
];

export function AuditLogTable() {
  return (
    <DataTable
      columns={columns}
      data={mockAuditLogs}
      emptyState={<EmptyState icon={FileSearch} title="No audit events" />}
    />
  );
}
