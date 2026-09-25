"use client";

import { Landmark, TrendingUp, Wallet, PiggyBank } from "lucide-react";

import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { FinancialMetric } from "@/components/financial/financial-metric";
import { Money } from "@/components/financial/money";
import { Percentage } from "@/components/financial/percentage";
import { usePortfolioSummary } from "../hooks/use-portfolio";

const EMPTY_SUMMARY = {
  totalValue: "0",
  totalCostBasis: "0",
  totalUnrealizedPnl: "0",
  totalUnrealizedPnlPercent: "0",
  cashBalance: "0",
  investedValue: "0",
  ytdIncome: "0",
};

export function PortfolioSummaryCards() {
  const { data, isPending } = usePortfolioSummary();

  if (isPending) {
    return (
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <Card key={i}>
            <CardContent className="pt-6">
              <Skeleton className="h-16 w-full" />
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  const summary = data ?? EMPTY_SUMMARY;

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      <Card className="border-primary/20 bg-primary/[0.03]">
        <CardContent className="pt-6">
          <FinancialMetric
            icon={Wallet}
            emphasis
            label="Total Portfolio Value"
            value={<Money value={summary.totalValue} />}
            trend={<Percentage value={summary.totalUnrealizedPnlPercent} colorize />}
          />
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric
            icon={TrendingUp}
            label="Unrealized P&L"
            value={
              <Money
                value={summary.totalUnrealizedPnl}
                className={Number(summary.totalUnrealizedPnl) >= 0 ? "text-success" : "text-danger"}
              />
            }
            hint={`Cost basis ${new Intl.NumberFormat("en-NG", { style: "currency", currency: "NGN", maximumFractionDigits: 0 }).format(Number(summary.totalCostBasis))}`}
          />
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric
            icon={Landmark}
            label="Cash Balance"
            value={<Money value={summary.cashBalance} />}
          />
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric
            icon={PiggyBank}
            label="YTD Income"
            value={<Money value={summary.ytdIncome} />}
          />
        </CardContent>
      </Card>
    </div>
  );
}
