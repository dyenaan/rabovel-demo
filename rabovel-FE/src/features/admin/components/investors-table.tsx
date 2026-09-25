"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { CheckCircle2, Users as UsersIcon, XCircle } from "lucide-react";
import { toast } from "sonner";

import { AdminActionMenu } from "@/components/shared/admin-action-menu";
import { AdminFilters } from "@/components/shared/admin-filters";
import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatDate } from "@/lib/formatters";
import { mockInvestors } from "@/mocks/investors.mock";
import type { Investor } from "@/types";

export function InvestorsTable() {
  const [investors, setInvestors] = useState<Investor[]>(mockInvestors);
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState("ALL");

  const filtered = useMemo(
    () =>
      investors.filter((inv) => {
        const matchesSearch =
          !search ||
          inv.fullName.toLowerCase().includes(search.toLowerCase()) ||
          inv.email.toLowerCase().includes(search.toLowerCase());
        const matchesStatus = statusFilter === "ALL" || inv.kycStatus === statusFilter;
        return matchesSearch && matchesStatus;
      }),
    [investors, search, statusFilter],
  );

  function updateKyc(investorId: string, status: Investor["kycStatus"]) {
    setInvestors((prev) => prev.map((i) => (i.investorId === investorId ? { ...i, kycStatus: status } : i)));
    toast.success(`KYC status updated to ${status}.`);
  }

  const columns: DataTableColumn<Investor>[] = [
    {
      accessorKey: "fullName",
      header: "Investor",
      cell: ({ row }) => (
        <Link href={`/admin/investors/${row.original.investorId}`} className="hover:underline">
          <p className="font-medium text-foreground">{row.original.fullName}</p>
          <p className="text-xs text-muted-foreground">{row.original.email}</p>
        </Link>
      ),
    },
    { accessorKey: "accountType", header: "Type" },
    { accessorKey: "jurisdiction", header: "Jurisdiction" },
    {
      accessorKey: "kycStatus",
      header: "KYC",
      cell: ({ row }) => <StatusBadge status={row.original.kycStatus} />,
    },
    {
      accessorKey: "eligibilityStatus",
      header: "Eligibility",
      cell: ({ row }) => <StatusBadge status={row.original.eligibilityStatus} />,
    },
    {
      accessorKey: "accountStatus",
      header: "Account",
      cell: ({ row }) => <StatusBadge status={row.original.accountStatus} />,
    },
    {
      accessorKey: "createdAt",
      header: "Joined",
      cell: ({ row }) => <span className="text-xs text-muted-foreground">{formatDate(row.original.createdAt)}</span>,
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => (
        <AdminActionMenu
          actions={[
            {
              label: "Approve KYC",
              icon: CheckCircle2,
              onSelect: () => updateKyc(row.original.investorId, "VERIFIED"),
            },
            {
              label: "Reject KYC",
              icon: XCircle,
              destructive: true,
              onSelect: () => updateKyc(row.original.investorId, "REJECTED"),
            },
          ]}
        />
      ),
    },
  ];

  return (
    <div className="space-y-4">
      <AdminFilters
        search={search}
        onSearchChange={setSearch}
        searchPlaceholder="Search investors…"
        statusValue={statusFilter}
        onStatusChange={setStatusFilter}
        statusOptions={[
          { label: "All statuses", value: "ALL" },
          { label: "Verified", value: "VERIFIED" },
          { label: "Pending", value: "PENDING" },
          { label: "Requires Action", value: "REQUIRES_ACTION" },
          { label: "Rejected", value: "REJECTED" },
        ]}
      />
      <DataTable
        columns={columns}
        data={filtered}
        emptyState={<EmptyState icon={UsersIcon} title="No investors found" />}
      />
    </div>
  );
}
