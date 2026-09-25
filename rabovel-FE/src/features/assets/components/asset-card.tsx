import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Money } from "@/components/financial/money";
import { Percentage } from "@/components/financial/percentage";
import { StatusBadge } from "@/components/shared/status-badge";
import type { Asset } from "@/types";

export const ASSET_CLASS_LABEL: Record<Asset["assetClass"], string> = {
  TREASURY: "Treasury",
  PRIVATE_CREDIT: "Private Credit",
  REAL_ESTATE: "Real Estate",
  INFRASTRUCTURE: "Infrastructure",
  COMMODITY: "Commodity",
  EQUITY: "Growth Equity",
};

export function AssetCard({ asset }: { asset: Asset }) {
  return (
    <Link href={`/assets/${asset.assetId}`}>
      <Card className="card-interactive h-full focus-visible:-translate-y-0.5 focus-visible:border-primary/30 focus-visible:shadow-md">
        <CardHeader>
          <div className="flex items-center justify-between">
            <Badge variant="secondary">{ASSET_CLASS_LABEL[asset.assetClass]}</Badge>
            <StatusBadge status={asset.lifecycleStatus} />
          </div>
          <h3 className="mt-2 text-sm font-semibold text-foreground">{asset.name}</h3>
          <p className="text-xs text-muted-foreground">
            {asset.symbol} · {asset.issuerName}
          </p>
        </CardHeader>
        <CardContent className="space-y-3 pb-6">
          <p className="text-sm text-muted-foreground line-clamp-2">{asset.description}</p>
          <div className="grid grid-cols-3 gap-2 border-t pt-3 text-center">
            <div>
              <p className="text-[10px] text-muted-foreground uppercase">NAV</p>
              <Money value={asset.nav} className="text-sm font-medium" />
            </div>
            <div>
              <p className="text-[10px] text-muted-foreground uppercase">Yield</p>
              <Percentage value={asset.yield} signed={false} className="text-sm font-medium" />
            </div>
            <div>
              <p className="text-[10px] text-muted-foreground uppercase">Min.</p>
              <Money value={asset.minimumInvestment} compact className="text-sm font-medium" />
            </div>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}
