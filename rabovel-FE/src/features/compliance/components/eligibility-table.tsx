"use client";

import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { UserCheck } from "lucide-react";
import { mockInvestors } from "@/mocks/investors.mock";
import type { Investor } from "@/types";

const columns: DataTableColumn<Investor>[] = [
  { accessorKey: "fullName", header: "Investor" },
  { accessorKey: "jurisdiction", header: "Jurisdiction" },
  { accessorKey: "accountType", header: "Type" },
  {
    accessorKey: "eligibilityStatus",
    header: "Eligibility",
    cell: ({ row }) => <StatusBadge status={row.original.eligibilityStatus} />,
  },
];

export function EligibilityTable() {
  return (
    <DataTable
      columns={columns}
      data={mockInvestors}
      emptyState={<EmptyState icon={UserCheck} title="No investors" />}
    />
  );
}
