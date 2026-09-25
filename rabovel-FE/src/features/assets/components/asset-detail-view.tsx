"use client";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { EmptyState } from "@/components/shared/empty-state";
import { ErrorState } from "@/components/shared/error-state";
import { LoadingState } from "@/components/shared/loading-state";
import { Building2 } from "lucide-react";

import { useAsset } from "../hooks/use-assets";
import { AssetDetailHeader } from "./asset-detail-header";
import { AssetDocumentsList } from "./asset-documents-list";
import { NavHistoryChart } from "./nav-history-chart";

export function AssetDetailView({ assetId }: { assetId: string }) {
  const { data: asset, isPending, isError, refetch } = useAsset(assetId);

  if (isPending) return <LoadingState label="Loading asset…" />;
  if (isError) return <ErrorState onRetry={() => refetch()} />;
  if (!asset) {
    return (
      <EmptyState
        icon={Building2}
        title="Asset not found"
        description="This asset may have been delisted or the link is incorrect."
      />
    );
  }

  return (
    <div className="space-y-6">
      <AssetDetailHeader asset={asset} />

      <Tabs defaultValue="overview">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="performance">NAV Performance</TabsTrigger>
          <TabsTrigger value="documents">Documents</TabsTrigger>
        </TabsList>
        <TabsContent value="overview" className="pt-4">
          <p className="max-w-3xl text-sm leading-relaxed text-muted-foreground">
            {asset.description}
          </p>
          <dl className="mt-6 grid grid-cols-2 gap-4 sm:grid-cols-3">
            <div>
              <dt className="text-xs text-muted-foreground uppercase">Settlement Currency</dt>
              <dd className="text-sm font-medium text-foreground">{asset.settlementCurrency}</dd>
            </div>
            <div>
              <dt className="text-xs text-muted-foreground uppercase">Blockchain</dt>
              <dd className="text-sm font-medium text-foreground">{asset.blockchain ?? "—"}</dd>
            </div>
            <div>
              <dt className="text-xs text-muted-foreground uppercase">Token Standard</dt>
              <dd className="text-sm font-medium text-foreground">{asset.tokenStandard ?? "—"}</dd>
            </div>
            <div>
              <dt className="text-xs text-muted-foreground uppercase">Inception Date</dt>
              <dd className="text-sm font-medium text-foreground">{asset.inceptionDate ?? "—"}</dd>
            </div>
            {asset.maturityDate && (
              <div>
                <dt className="text-xs text-muted-foreground uppercase">Maturity Date</dt>
                <dd className="text-sm font-medium text-foreground">{asset.maturityDate}</dd>
              </div>
            )}
            {asset.contractAddress && (
              <div>
                <dt className="text-xs text-muted-foreground uppercase">Contract Address</dt>
                <dd className="font-mono text-xs font-medium text-foreground">
                  {asset.contractAddress}
                </dd>
              </div>
            )}
          </dl>
        </TabsContent>
        <TabsContent value="performance" className="pt-4">
          <NavHistoryChart assetId={assetId} />
        </TabsContent>
        <TabsContent value="documents" className="pt-4">
          <AssetDocumentsList documents={asset.documents} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
