"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { mockAssets } from "@/mocks/assets.mock";
import { useReserveStatus } from "../hooks/use-primary-market";

export function ReserveStatus() {
  const { data, isPending } = useReserveStatus();

  return (
    <Card>
      <CardHeader>
        <CardTitle>Reserve Coverage</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4 pb-6">
        {isPending
          ? Array.from({ length: 4 }).map((_, i) => <Skeleton key={i} className="h-10 w-full" />)
          : data?.map((reserve) => {
              const asset = mockAssets.find((a) => a.assetId === reserve.assetId);
              return (
                <div key={reserve.assetId}>
                  <div className="mb-1 flex items-center justify-between text-sm">
                    <span className="text-foreground">{asset?.symbol ?? reserve.assetId}</span>
                    <span className="font-tabular text-muted-foreground">{reserve.coveragePercent}%</span>
                  </div>
                  <Progress value={Number(reserve.coveragePercent)} />
                </div>
              );
            })}
      </CardContent>
    </Card>
  );
}
