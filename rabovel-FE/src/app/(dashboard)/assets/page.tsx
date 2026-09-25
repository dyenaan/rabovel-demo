"use client";

import { useMemo, useState } from "react";
import { Building2 } from "lucide-react";

import { PageContainer } from "@/components/layout/page-container";
import { EmptyState } from "@/components/shared/empty-state";
import { ErrorState } from "@/components/shared/error-state";
import { PageHeader } from "@/components/shared/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import { AssetCard } from "@/features/assets/components/asset-card";
import { AssetFilters } from "@/features/assets/components/asset-filters";
import { useAssets } from "@/features/assets/hooks/use-assets";
import type { AssetClass } from "@/types";

export default function AssetsMarketplacePage() {
  const { data, isPending, isError, refetch } = useAssets();
  const [search, setSearch] = useState("");
  const [selectedClass, setSelectedClass] = useState<AssetClass | "ALL">("ALL");

  const filtered = useMemo(() => {
    if (!data) return [];
    return data.filter((asset) => {
      const matchesClass = selectedClass === "ALL" || asset.assetClass === selectedClass;
      const matchesSearch =
        !search ||
        asset.name.toLowerCase().includes(search.toLowerCase()) ||
        asset.symbol.toLowerCase().includes(search.toLowerCase());
      return matchesClass && matchesSearch;
    });
  }, [data, search, selectedClass]);

  return (
    <PageContainer>
      <PageHeader
        title="Assets"
        description="Browse tokenized assets available for subscription and secondary trading."
      />

      <div className="space-y-6">
        <AssetFilters
          search={search}
          onSearchChange={setSearch}
          selectedClass={selectedClass}
          onSelectedClassChange={setSelectedClass}
        />

        {isError ? (
          <ErrorState onRetry={() => refetch()} />
        ) : isPending ? (
          <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-64 w-full" />
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <EmptyState icon={Building2} title="No assets match your filters" />
        ) : (
          <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {filtered.map((asset) => (
              <AssetCard key={asset.assetId} asset={asset} />
            ))}
          </div>
        )}
      </div>
    </PageContainer>
  );
}
