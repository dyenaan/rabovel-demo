"use client";

import { ShieldCheck } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/shared/empty-state";
import { ErrorState } from "@/components/shared/error-state";
import { LoadingState } from "@/components/shared/loading-state";
import { StatusBadge } from "@/components/shared/status-badge";
import { Money } from "@/components/financial/money";
import { formatDateTime, truncateAddress } from "@/lib/formatters";
import { useSettlement } from "../hooks/use-settlements";
import { deriveFinalityState } from "../utils/finality";
import { FinalityStatus } from "./finality-status";
import { SettlementTimeline } from "./settlement-timeline";

export function SettlementDetailView({ settlementId }: { settlementId: string }) {
  const { data: settlement, isPending, isError, refetch } = useSettlement(settlementId);

  if (isPending) return <LoadingState label="Loading settlement…" />;
  if (isError) return <ErrorState onRetry={() => refetch()} />;
  if (!settlement) return <EmptyState icon={ShieldCheck} title="Settlement not found" />;

  const finalityState = deriveFinalityState(settlement.status);

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center gap-3">
        <h1 className="text-xl font-semibold text-foreground">{settlement.settlementId}</h1>
        <StatusBadge status={settlement.status} />
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Lifecycle</CardTitle>
          </CardHeader>
          <CardContent className="pb-6">
            <SettlementTimeline status={settlement.status} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Blockchain Finality</CardTitle>
          </CardHeader>
          <CardContent className="pb-6">
            {finalityState ? (
              <FinalityStatus state={finalityState} />
            ) : (
              <p className="text-sm text-muted-foreground">
                Not yet submitted on-chain — settlement is still in pre-submission processing.
              </p>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Legs & Routing</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 pb-6 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Asset Leg</span>
              <span className="font-tabular">
                {settlement.assetLeg.amount} {settlement.assetLeg.assetOrCurrency}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Cash Leg</span>
              <Money value={settlement.cashLeg.amount} currency={settlement.cashLeg.assetOrCurrency} />
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Custody Route</span>
              <span>{settlement.custodyRoute ?? "—"}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Chain ID</span>
              <span className="font-mono text-xs">{settlement.chainId ?? "—"}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Finality Policy</span>
              <span>{settlement.finalityPolicy ?? "—"}</span>
            </div>
            {settlement.transactionHash && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Transaction</span>
                <span className="font-mono text-xs">{truncateAddress(settlement.transactionHash, 6)}</span>
              </div>
            )}
            <div className="flex justify-between border-t pt-3">
              <span className="text-muted-foreground">Created</span>
              <span>{formatDateTime(settlement.createdAt)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Updated</span>
              <span>{formatDateTime(settlement.updatedAt)}</span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
