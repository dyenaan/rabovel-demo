"use client";

import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Money } from "@/components/financial/money";
import { Percentage } from "@/components/financial/percentage";
import { useAssets } from "@/features/assets/hooks/use-assets";

export function NavSummary() {
  const { data, isPending } = useAssets();
  const openForSubscription = (data ?? []).filter((a) => a.lifecycleStatus === "SUBSCRIPTION_OPEN" || a.lifecycleStatus === "ACTIVE");

  return (
    <Card>
      <CardHeader>
        <CardTitle>Open for Subscription</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3 pb-6">
        {isPending
          ? Array.from({ length: 3 }).map((_, i) => <Skeleton key={i} className="h-16 w-full" />)
          : openForSubscription.map((asset) => (
              <div key={asset.assetId} className="flex items-center justify-between rounded-md border p-3">
                <div>
                  <p className="text-sm font-medium text-foreground">{asset.name}</p>
                  <div className="mt-1 flex items-center gap-3 text-xs text-muted-foreground">
                    <span>
                      NAV <Money value={asset.nav} className="font-medium text-foreground" />
                    </span>
                    <span>
                      Yield <Percentage value={asset.yield} signed={false} className="font-medium text-foreground" />
                    </span>
                    {asset.lifecycleStatus === "SUBSCRIPTION_OPEN" && (
                      <Badge variant="success" className="text-[10px]">
                        Open
                      </Badge>
                    )}
                  </div>
                </div>
                <Button size="sm" asChild>
                  <Link href={`/subscribe?assetId=${asset.assetId}`}>Subscribe</Link>
                </Button>
              </div>
            ))}
      </CardContent>
    </Card>
  );
}
