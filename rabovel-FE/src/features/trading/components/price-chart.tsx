"use client";

import { useState } from "react";
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { formatDate, formatPrice } from "@/lib/formatters";
import { usePriceHistory } from "@/features/markets/hooks/use-markets";

const RANGES = ["1D", "1W", "1M"] as const;

export function PriceChart({ marketId }: { marketId: string }) {
  const [range, setRange] = useState<(typeof RANGES)[number]>("1D");
  const { data, isPending } = usePriceHistory(marketId, range);

  const chartData = (data ?? []).map((point) => ({
    timestamp: point.timestamp,
    close: Number(point.close),
    volume: Number(point.volume),
  }));

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <p className="text-sm font-medium text-foreground">Price Chart</p>
        <Tabs value={range} onValueChange={(v) => setRange(v as typeof range)}>
          <TabsList>
            {RANGES.map((r) => (
              <TabsTrigger key={r} value={r} className="text-xs">
                {r}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      </div>

      {isPending ? (
        <Skeleton className="h-72 w-full" />
      ) : (
        <div>
          <ResponsiveContainer width="100%" height={240}>
            <AreaChart data={chartData} margin={{ left: 0, right: 8, top: 8, bottom: 0 }}>
              <defs>
                <linearGradient id="priceClose" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="var(--color-chart-1)" stopOpacity={0.35} />
                  <stop offset="100%" stopColor="var(--color-chart-1)" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid vertical={false} stroke="var(--color-border)" strokeDasharray="3 3" />
              <XAxis
                dataKey="timestamp"
                tickFormatter={(value) =>
                  formatDate(value, { hour: "2-digit", minute: "2-digit" })
                }
                tickLine={false}
                axisLine={false}
                tick={{ fontSize: 11, fill: "var(--color-muted-foreground)" }}
                minTickGap={50}
              />
              <YAxis
                domain={["auto", "auto"]}
                tickFormatter={(value) => formatPrice(value, 0)}
                tickLine={false}
                axisLine={false}
                width={48}
                tick={{ fontSize: 11, fill: "var(--color-muted-foreground)" }}
              />
              <Tooltip
                formatter={(value) => formatPrice(value as number)}
                labelFormatter={(value) => formatDate(value as string, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
                contentStyle={{
                  background: "var(--color-popover)",
                  border: "1px solid var(--color-border)",
                  borderRadius: 8,
                  fontSize: 12,
                }}
              />
              <Area
                type="monotone"
                dataKey="close"
                stroke="var(--color-chart-1)"
                strokeWidth={2}
                fill="url(#priceClose)"
              />
            </AreaChart>
          </ResponsiveContainer>
          <ResponsiveContainer width="100%" height={60}>
            <BarChart data={chartData} margin={{ left: 0, right: 8, top: 0, bottom: 0 }}>
              <XAxis dataKey="timestamp" hide />
              <YAxis hide />
              <Bar dataKey="volume" fill="var(--color-muted-foreground)" opacity={0.35} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  );
}
