"use client";

import { useMemo } from "react";
import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Decimal,
  formatCompactMoney,
  formatMoney,
  formatPercentage,
  formatQuantity,
} from "@/lib/formatters";
import { useHoldings } from "../hooks/use-portfolio";

const COLORS = [
  "var(--color-chart-1)",
  "var(--color-chart-2)",
  "var(--color-chart-3)",
  "var(--color-chart-4)",
  "var(--color-chart-5)",
];

export function AssetAllocationChart() {
  const { data, isPending } = useHoldings();

  const chartData = useMemo(
    () => (data ?? []).map((holding) => ({ ...holding, allocationPercent: Number(holding.allocationPercent) })),
    [data],
  );

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>Asset Allocation</CardTitle>
      </CardHeader>
      <CardContent className="flex h-full flex-col justify-center pb-6">
        {isPending ? (
          <Skeleton className="h-64 w-full" />
        ) : !data || data.length === 0 ? (
          <div className="flex min-h-64 flex-col items-center justify-center gap-3 rounded-lg border border-dashed px-6 text-center">
            <div className="flex size-24 items-center justify-center rounded-full border-8 border-muted">
              <span className="font-tabular text-sm font-semibold text-foreground">
                {formatCompactMoney("0")}
              </span>
            </div>
            <div>
              <p className="text-sm font-medium text-foreground">No holdings yet</p>
              <p className="mt-1 text-sm text-muted-foreground">
                Assets you purchase will appear here.
              </p>
            </div>
          </div>
        ) : (
          <div className="flex flex-col items-center gap-4">
            <div className="relative w-full max-w-[220px]">
              <ResponsiveContainer width="100%" height={220}>
                <PieChart>
                  <Pie
                    data={chartData}
                    dataKey="allocationPercent"
                    nameKey="symbol"
                    innerRadius={55}
                    outerRadius={90}
                    paddingAngle={2}
                  >
                    {chartData.map((entry, index) => (
                      <Cell
                        key={entry.assetId}
                        fill={COLORS[index % COLORS.length]}
                        stroke="none"
                      />
                    ))}
                  </Pie>
                  <Tooltip
                    formatter={(value) => formatPercentage(value as number, { signed: false })}
                    contentStyle={{
                      background: "var(--color-popover)",
                      border: "1px solid var(--color-border)",
                      borderRadius: 8,
                      fontSize: 12,
                    }}
                  />
                </PieChart>
              </ResponsiveContainer>
              <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
                <span className="text-[10px] font-medium tracking-wide text-muted-foreground uppercase">
                  Total
                </span>
                <span className="font-tabular text-sm font-semibold text-foreground">
                  {formatCompactMoney(
                    data
                      .reduce((sum, holding) => sum.plus(new Decimal(holding.marketValue)), new Decimal(0))
                      .toString(),
                  )}
                </span>
              </div>
            </div>
            <ul className="w-full divide-y">
              {data.map((holding, index) => (
                <li key={holding.assetId} className="flex items-center justify-between gap-3 py-3 text-sm">
                  <span className="flex min-w-0 items-center gap-2">
                    <span
                      className="size-2.5 shrink-0 rounded-full"
                      style={{ background: COLORS[index % COLORS.length] }}
                    />
                    <span className="min-w-0">
                      <span className="block truncate font-medium text-foreground">
                        {holding.assetName}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {formatQuantity(holding.quantity)} {holding.symbol}
                      </span>
                    </span>
                  </span>
                  <span className="shrink-0 text-right font-tabular">
                    <span className="block font-medium text-foreground">
                      {formatMoney(holding.marketValue)}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {formatPercentage(holding.allocationPercent, { signed: false })}
                    </span>
                  </span>
                </li>
              ))}
            </ul>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
