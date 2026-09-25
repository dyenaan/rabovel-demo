"use client";

import { useMemo, useState } from "react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { formatCompactMoney, formatDate, formatMoney } from "@/lib/formatters";
import { usePortfolioPerformance } from "../hooks/use-portfolio";

const RANGES = ["1M", "3M", "6M", "1Y"] as const;

export function PortfolioPerformanceChart() {
  const [range, setRange] = useState<(typeof RANGES)[number]>("6M");
  const { data, isPending } = usePortfolioPerformance(range);

  const chartData = useMemo(
    () =>
      (data ?? []).map((point) => ({
        date: point.date,
        value: Number(point.portfolioValue),
        benchmark: point.benchmarkValue ? Number(point.benchmarkValue) : undefined,
      })),
    [data],
  );

  const hasBenchmark = chartData.some((point) => point.benchmark !== undefined);

  return (
    <Card>
      <CardHeader className="flex-row flex-wrap items-center justify-between gap-3 space-y-0">
        <div className="flex items-center gap-4">
          <CardTitle>Portfolio Performance</CardTitle>
          <div className="flex items-center gap-3 text-xs text-muted-foreground">
            <span className="flex items-center gap-1.5">
              <span className="h-0.5 w-3 rounded-full bg-[var(--color-chart-1)]" />
              Portfolio
            </span>
            {hasBenchmark && (
              <span className="flex items-center gap-1.5">
                <span
                  className="h-0.5 w-4"
                  style={{
                    backgroundImage:
                      "repeating-linear-gradient(90deg, var(--color-chart-3) 0 3px, transparent 3px 5px)",
                  }}
                />
                Benchmark
              </span>
            )}
          </div>
        </div>
        <Tabs value={range} onValueChange={(v) => setRange(v as typeof range)}>
          <TabsList>
            {RANGES.map((r) => (
              <TabsTrigger key={r} value={r} className="text-xs">
                {r}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      </CardHeader>
      <CardContent className="pb-6">
        {isPending ? (
          <Skeleton className="h-64 w-full" />
        ) : (
          <ResponsiveContainer width="100%" height={260}>
            <AreaChart data={chartData} margin={{ left: 0, right: 8, top: 8, bottom: 0 }}>
              <defs>
                <linearGradient id="portfolioValue" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="var(--color-chart-1)" stopOpacity={0.35} />
                  <stop offset="100%" stopColor="var(--color-chart-1)" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid vertical={false} stroke="var(--color-border)" strokeDasharray="3 3" />
              <XAxis
                dataKey="date"
                tickFormatter={(value) => formatDate(value, { month: "short", day: "numeric" })}
                tickLine={false}
                axisLine={false}
                tick={{ fontSize: 11, fill: "var(--color-muted-foreground)" }}
                minTickGap={40}
              />
              <YAxis
                domain={[(dataMin: number) => Math.floor(dataMin * 0.97), (dataMax: number) => Math.ceil(dataMax * 1.03)]}
                tickFormatter={(value) => formatCompactMoney(value)}
                tickLine={false}
                axisLine={false}
                width={56}
                tick={{ fontSize: 11, fill: "var(--color-muted-foreground)" }}
              />
              <Tooltip
                formatter={(value) => formatMoney(value as number)}
                labelFormatter={(value) => formatDate(value as string)}
                contentStyle={{
                  background: "var(--color-popover)",
                  border: "1px solid var(--color-border)",
                  borderRadius: 8,
                  fontSize: 12,
                }}
              />
              <Area
                type="monotone"
                dataKey="value"
                stroke="var(--color-chart-1)"
                strokeWidth={2}
                fill="url(#portfolioValue)"
                name="Portfolio"
                activeDot={{ r: 4, strokeWidth: 0 }}
              />
              <Area
                type="monotone"
                dataKey="benchmark"
                stroke="var(--color-chart-3)"
                strokeWidth={1.5}
                strokeDasharray="4 4"
                fill="none"
                name="Benchmark"
              />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
