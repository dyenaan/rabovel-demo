import { Star } from "lucide-react";

import { Card, CardContent } from "@/components/ui/card";
import { StatusBadge } from "@/components/shared/status-badge";
import type { Wallet } from "@/types";
import { AddressDisplay } from "./address-display";
import { NetworkBadge } from "./network-badge";

export function WalletCard({ wallet }: { wallet: Wallet }) {
  return (
    <Card>
      <CardContent className="flex flex-col items-start justify-between gap-3 sm:flex-row sm:items-center pt-6">
        <div className="space-y-1.5">
          <div className="flex items-center gap-2">
            <p className="text-sm font-medium text-foreground">{wallet.label}</p>
            {wallet.isPrimary && <Star className="size-3.5 fill-warning text-warning" />}
          </div>
          <details className="text-xs">
            <summary className="cursor-pointer text-muted-foreground">Connection details</summary>
            <div className="mt-2 flex flex-wrap items-center gap-2"><NetworkBadge blockchain={wallet.blockchain} />
            <AddressDisplay address={wallet.address} /></div>
          </details>
        </div>
        <StatusBadge status={wallet.status} />
      </CardContent>
    </Card>
  );
}
