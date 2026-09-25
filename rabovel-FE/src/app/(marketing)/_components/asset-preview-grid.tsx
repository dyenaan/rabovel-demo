import Link from "next/link";
import { ArrowRight } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Money } from "@/components/financial/money";
import { Percentage } from "@/components/financial/percentage";
import { ASSET_CLASS_LABEL } from "@/features/assets/components/asset-card";
import { mockAssets } from "@/mocks/assets.mock";

export function AssetPreviewGrid() {
  const featured = mockAssets.filter((asset) => asset.lifecycleStatus !== "PENDING_APPROVAL").slice(0, 4);

  return (
    <section className="border-b bg-secondary/30">
      <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h2 className="text-3xl font-semibold tracking-tight text-foreground">
              Diversified tokenized assets
            </h2>
            <p className="mt-3 max-w-xl text-muted-foreground">
              From short-duration treasuries to private credit and real assets — each fund is
              structured, audited, and issued on-chain under institutional custody.
            </p>
          </div>
          <Button variant="outline" asChild className="w-fit">
            <Link href="/assets">
              View all assets
              <ArrowRight className="size-4" />
            </Link>
          </Button>
        </div>

        <div className="mt-10 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {featured.map((asset) => (
            <Card key={asset.assetId} className="card-interactive">
              <CardHeader>
                <Badge variant="secondary" className="w-fit">
                  {ASSET_CLASS_LABEL[asset.assetClass]}
                </Badge>
                <h3 className="mt-2 text-sm font-semibold text-foreground">{asset.name}</h3>
                <p className="text-xs text-muted-foreground">{asset.symbol}</p>
              </CardHeader>
              <CardContent className="space-y-3 pb-6">
                <div className="flex items-baseline justify-between">
                  <span className="text-xs text-muted-foreground">NAV</span>
                  <Money value={asset.nav} className="text-sm font-medium" />
                </div>
                <div className="flex items-baseline justify-between">
                  <span className="text-xs text-muted-foreground">Target Yield</span>
                  <Percentage value={asset.yield} signed={false} className="text-sm font-medium" />
                </div>
                <div className="flex items-baseline justify-between">
                  <span className="text-xs text-muted-foreground">Minimum</span>
                  <Money value={asset.minimumInvestment} compact className="text-sm font-medium" />
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
}
