import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { Card, CardContent } from "@/components/ui/card";
import { FinancialMetric } from "@/components/financial/financial-metric";
import { AddressDisplay } from "@/features/custody/components/address-display";
import { NetworkBadge } from "@/features/custody/components/network-badge";
import { StatusBadge } from "@/components/shared/status-badge";
import { mockWallets } from "@/mocks/custody.mock";

export const metadata: Metadata = { title: "Custody — Admin" };

export default function AdminCustodyPage() {
  const networks = new Set(mockWallets.map((w) => w.blockchain)).size;
  const active = mockWallets.filter((w) => w.status === "ACTIVE").length;

  return (
    <PageContainer>
      <PageHeader title="Custody Operations" description="Platform-wide custody wallets and network coverage." />

      <div className="mb-6 grid gap-4 sm:grid-cols-3">
        <Card>
          <CardContent className="pt-6">
            <FinancialMetric label="Total Wallets" value={mockWallets.length} />
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

      <Card>
        <CardContent className="divide-y pt-6">
          {mockWallets.map((wallet) => (
            <div key={wallet.walletId} className="flex items-center justify-between py-3 first:pt-0 last:pb-0">
              <div className="flex items-center gap-3">
                <NetworkBadge blockchain={wallet.blockchain} />
                <div>
                  <p className="text-sm font-medium text-foreground">{wallet.label}</p>
                  <AddressDisplay address={wallet.address} />
                </div>
              </div>
              <StatusBadge status={wallet.status} />
            </div>
          ))}
        </CardContent>
      </Card>
    </PageContainer>
  );
}
