"use client";

import { Card, CardContent } from "@/components/ui/card";
import { FinancialMetric } from "@/components/financial/financial-metric";
import { useWallets } from "../hooks/use-wallets";

export function CustodyOverview() {
  const { data } = useWallets();
  const wallets = data ?? [];
  const networks = new Set(wallets.map((w) => w.blockchain)).size;
  const active = wallets.filter((w) => w.status === "ACTIVE").length;

  return (
    <div className="grid gap-4 sm:grid-cols-3">
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric label="Bound Wallets" value={wallets.length} />
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric label="Active" value={active} />
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-6">
          <FinancialMetric label="Networks" value={networks} />
        </CardContent>
      </Card>
    </div>
  );
}
