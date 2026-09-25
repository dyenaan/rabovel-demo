"use client";

import { useMemo, useState } from "react";
import { Building2, CheckCircle2, PauseCircle, XCircle } from "lucide-react";
import { toast } from "sonner";

import { AdminActionMenu } from "@/components/shared/admin-action-menu";
import { AdminFilters } from "@/components/shared/admin-filters";
import { DataTable, type DataTableColumn } from "@/components/shared/data-table";
import { EmptyState } from "@/components/shared/empty-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { Money } from "@/components/financial/money";
import { mockAssets } from "@/mocks/assets.mock";
import type { Asset } from "@/types";

export function AdminAssetsTable() {
  const [assets, setAssets] = useState<Asset[]>(mockAssets);
  const [search, setSearch] = useState("");

  const filtered = useMemo(
    () =>
      assets.filter(
        (a) =>
          !search ||
          a.name.toLowerCase().includes(search.toLowerCase()) ||
          a.symbol.toLowerCase().includes(search.toLowerCase()),
      ),
    [assets, search],
  );

  function updateStatus(assetId: string, status: Asset["lifecycleStatus"]) {
    setAssets((prev) => prev.map((a) => (a.assetId === assetId ? { ...a, lifecycleStatus: status } : a)));
    toast.success(`Asset lifecycle status updated to ${status}.`);
  }

  const columns: DataTableColumn<Asset>[] = [
    {
      accessorKey: "name",
      header: "Asset",
      cell: ({ row }) => (
        <div>
          <p className="font-medium text-foreground">{row.original.name}</p>
          <p className="text-xs text-muted-foreground">{row.original.symbol} · {row.original.issuerName}</p>
        </div>
      ),
    },
    { accessorKey: "assetClass", header: "Class" },
    {
      accessorKey: "nav",
      header: "NAV",
      cell: ({ row }) => <Money value={row.original.nav} />,
    },
    {
      accessorKey: "totalOutstanding",
      header: "Outstanding",
      cell: ({ row }) => <Money value={row.original.totalOutstanding} compact />,
    },
    {
      accessorKey: "lifecycleStatus",
      header: "Status",
      cell: ({ row }) => <StatusBadge status={row.original.lifecycleStatus} />,
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => (
        <AdminActionMenu
          actions={[
            { label: "Approve", icon: CheckCircle2, onSelect: () => updateStatus(row.original.assetId, "ACTIVE") },
            { label: "Suspend", icon: PauseCircle, onSelect: () => updateStatus(row.original.assetId, "SUSPENDED") },
            { label: "Delist", icon: XCircle, destructive: true, onSelect: () => updateStatus(row.original.assetId, "DELISTED") },
          ]}
        />
      ),
    },
  ];

  return (
    <div className="space-y-4">
      <AdminFilters search={search} onSearchChange={setSearch} searchPlaceholder="Search assets…" />
      <DataTable
        columns={columns}
        data={filtered}
        emptyState={<EmptyState icon={Building2} title="No assets found" />}
      />
    </div>
  );
}
