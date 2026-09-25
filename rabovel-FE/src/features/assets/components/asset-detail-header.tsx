import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Money } from "@/components/financial/money";
import { Percentage } from "@/components/financial/percentage";
import { StatusBadge } from "@/components/shared/status-badge";
import type { Asset } from "@/types";
import { ASSET_CLASS_LABEL } from "./asset-card";

export function AssetDetailHeader({ asset }: { asset: Asset }) {
  return (
    <div className="flex flex-col gap-6 border-b pb-6 lg:flex-row lg:items-end lg:justify-between">
      <div>
        <div className="flex items-center gap-2">
          <Badge variant="secondary">{ASSET_CLASS_LABEL[asset.assetClass]}</Badge>
          <StatusBadge status={asset.lifecycleStatus} />
        </div>
        <h1 className="mt-2 text-2xl font-semibold text-foreground">{asset.name}</h1>
        <p className="text-sm text-muted-foreground">
          {asset.symbol} · Issued by {asset.issuerName}
        </p>
      </div>

      <div className="grid grid-cols-2 gap-6 sm:grid-cols-4">
        <div>
          <p className="text-xs text-muted-foreground uppercase">NAV</p>
          <Money value={asset.nav} className="text-lg font-semibold" />
        </div>
        <div>
          <p className="text-xs text-muted-foreground uppercase">Market Price</p>
          <Money value={asset.marketPrice} className="text-lg font-semibold" />
        </div>
        <div>
          <p className="text-xs text-muted-foreground uppercase">Target Yield</p>
          <Percentage value={asset.yield} signed={false} className="text-lg font-semibold" />
        </div>
        <div>
          <p className="text-xs text-muted-foreground uppercase">Minimum</p>
          <Money value={asset.minimumInvestment} compact className="text-lg font-semibold" />
        </div>
      </div>

      <div className="flex gap-2">
        <Button variant="outline" asChild>
          <Link href={`/markets?asset=${asset.symbol}`}>View Market</Link>
        </Button>
        <Button asChild>
          <Link href={`/subscribe?assetId=${asset.assetId}`}>Subscribe</Link>
        </Button>
      </div>
    </div>
  );
}
