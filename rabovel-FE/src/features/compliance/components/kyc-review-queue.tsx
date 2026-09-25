"use client";

import { useState } from "react";
import { CheckCircle2, ScanEye, XCircle } from "lucide-react";
import { toast } from "sonner";

import { AdminActionMenu } from "@/components/shared/admin-action-menu";
import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatDate } from "@/lib/formatters";
import { mockComplianceCases } from "@/mocks/compliance.mock";
import type { ComplianceCase } from "@/types";

export function KycReviewQueue() {
  const [cases, setCases] = useState<ComplianceCase[]>(
    mockComplianceCases.filter((c) => c.status === "PENDING" || c.status === "REQUIRES_ACTION"),
  );

  function decide(caseId: string, status: ComplianceCase["status"]) {
    setCases((prev) => prev.filter((c) => c.caseId !== caseId));
    toast.success(`Case ${caseId} marked ${status.toLowerCase()}.`);
  }

  const columns: DataTableColumn<ComplianceCase>[] = [
    {
      accessorKey: "investorName",
      header: "Investor",
      cell: ({ row }) => <span className="font-medium">{row.original.investorName}</span>,
    },
    {
      accessorKey: "reason",
      header: "Reason",
      cell: ({ row }) => row.original.reason?.replaceAll("_", " ") ?? "—",
    },
    {
      accessorKey: "status",
      header: "Status",
      cell: ({ row }) => <StatusBadge status={row.original.status} />,
    },
    {
      accessorKey: "openedAt",
      header: "Opened",
      cell: ({ row }) => <span className="text-xs text-muted-foreground">{formatDate(row.original.openedAt)}</span>,
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => (
        <AdminActionMenu
          actions={[
            { label: "Approve", icon: CheckCircle2, onSelect: () => decide(row.original.caseId, "VERIFIED") },
            { label: "Reject", icon: XCircle, destructive: true, onSelect: () => decide(row.original.caseId, "REJECTED") },
          ]}
        />
      ),
    },
  ];

  return (
    <DataTable
      columns={columns}
      data={cases}
      emptyState={<EmptyState icon={ScanEye} title="Review queue is empty" description="No pending KYC cases." />}
    />
  );
}
