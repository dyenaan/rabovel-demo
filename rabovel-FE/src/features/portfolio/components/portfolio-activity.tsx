"use client";

import {
  Activity as ActivityIcon,
  ArrowDownToLine,
  ArrowUpFromLine,
  FileText,
  ShieldCheck,
  TrendingUp,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/shared/empty-state";
import { LoadingState } from "@/components/shared/loading-state";
import { formatRelativeTime } from "@/lib/formatters";
import type { ActivityEvent } from "@/types";
import { usePortfolioActivity } from "../hooks/use-portfolio";

const ICONS: Record<ActivityEvent["type"], LucideIcon> = {
  ORDER_PLACED: TrendingUp,
  ORDER_FILLED: TrendingUp,
  SETTLEMENT_UPDATE: ShieldCheck,
  SUBSCRIPTION: ArrowDownToLine,
  REDEMPTION: ArrowUpFromLine,
  DEPOSIT: ArrowDownToLine,
  WITHDRAWAL: ArrowUpFromLine,
  KYC_UPDATE: ShieldCheck,
  DOCUMENT_PUBLISHED: FileText,
};

export function PortfolioActivity() {
  const { data, isPending } = usePortfolioActivity();

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>Recent Activity</CardTitle>
      </CardHeader>
      <CardContent className="pb-6">
        {isPending ? (
          <LoadingState label="Loading activity…" />
        ) : !data || data.length === 0 ? (
          <EmptyState
            icon={ActivityIcon}
            title="No recent activity"
            description="Completed purchases and account updates will appear here."
            className="py-10"
          />
        ) : (
          <ul className="space-y-4">
            {data.map((event) => {
              const Icon = ICONS[event.type];
              return (
                <li key={event.eventId} className="flex gap-3">
                  <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted">
                    <Icon className="size-4 text-muted-foreground" aria-hidden="true" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium text-foreground">{event.title}</p>
                    <p className="text-sm text-muted-foreground">{event.description}</p>
                    <p className="mt-0.5 text-xs text-muted-foreground/70">
                      {formatRelativeTime(event.timestamp)}
                    </p>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
