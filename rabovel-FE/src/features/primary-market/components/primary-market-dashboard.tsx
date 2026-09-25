"use client";

import Link from "next/link";
import { FileStack } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/shared/empty-state";
import { Money } from "@/components/financial/money";
import { StatusBadge } from "@/components/shared/status-badge";
import { formatDate, formatQuantity } from "@/lib/formatters";
import { useRedemptions, useSubscriptions } from "../hooks/use-primary-market";
import { NavSummary } from "./nav-summary";
import { ReserveStatus } from "./reserve-status";

export function PrimaryMarketDashboard() {
  const { data: subscriptions } = useSubscriptions();
  const { data: redemptions } = useRedemptions();

  return (
    <div className="space-y-6">
      <div className="flex gap-3">
        <Button asChild>
          <Link href="/subscribe">New Subscription</Link>
        </Button>
        <Button variant="outline" asChild>
          <Link href="/redeem">New Redemption</Link>
        </Button>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <NavSummary />
        <ReserveStatus />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Subscription & Redemption Activity</CardTitle>
        </CardHeader>
        <CardContent className="pb-6">
          {(!subscriptions || subscriptions.length === 0) && (!redemptions || redemptions.length === 0) ? (
            <EmptyState icon={FileStack} title="No activity yet" />
          ) : (
            <ul className="divide-y">
              {subscriptions?.map((s) => (
                <li key={s.subscriptionId} className="flex items-center justify-between py-3 first:pt-0">
                  <div>
                    <p className="text-sm font-medium text-foreground">Subscription — {s.assetName}</p>
                    <p className="text-xs text-muted-foreground">{formatDate(s.submittedAt)}</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <Money value={s.amount} currency={s.currency} />
                    <StatusBadge status={s.status} />
                  </div>
                </li>
              ))}
              {redemptions?.map((r) => (
                <li key={r.redemptionId} className="flex items-center justify-between py-3 last:pb-0">
                  <div>
                    <p className="text-sm font-medium text-foreground">Redemption — {r.assetName}</p>
                    <p className="text-xs text-muted-foreground">{formatDate(r.submittedAt)}</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="font-tabular text-sm">{formatQuantity(r.quantity)}</span>
                    <StatusBadge status={r.status} />
                  </div>
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
