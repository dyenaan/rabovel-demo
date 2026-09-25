"use client";

import { useMemo } from "react";
import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Decimal, formatCompactMoney, formatPercentage } from "@/lib/formatters";
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
        {isPending || !data ? (
          <Skeleton className="h-64 w-full" />
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
            <ul className="w-full space-y-2">
              {data.map((holding, index) => (
                <li key={holding.assetId} className="flex items-center justify-between text-sm">
                  <span className="flex items-center gap-2">
                    <span
                      className="size-2.5 rounded-full"
                      style={{ background: COLORS[index % COLORS.length] }}
                    />
                    {holding.symbol}
                  </span>
                  <span className="font-tabular text-muted-foreground">
                    {formatPercentage(holding.allocationPercent, { signed: false })}
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
