import type { Metadata } from "next";
import Link from "next/link";
import { AlertTriangle, Building2, ListOrdered, ScanEye, Users } from "lucide-react";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { Card, CardContent } from "@/components/ui/card";
import { FinancialMetric } from "@/components/financial/financial-metric";
import { Money } from "@/components/financial/money";
import { mockAssets } from "@/mocks/assets.mock";
import { mockComplianceCases } from "@/mocks/compliance.mock";
import { mockInvestors } from "@/mocks/investors.mock";
import { mockOrders } from "@/mocks/orders.mock";
import { mockReconciliationRecords } from "@/mocks/compliance.mock";
import { OPEN_ORDER_STATUSES } from "@/types";

export const metadata: Metadata = { title: "Admin Dashboard" };

export default function AdminDashboardPage() {
  // totalOutstanding is already expressed in settlement-currency (dollar) terms,
  // not unit count, so it is summed directly rather than multiplied by price.
  const totalAum = mockAssets.reduce((sum, asset) => sum + Number(asset.totalOutstanding ?? 0), 0);

  const openOrders = mockOrders.filter((o) => OPEN_ORDER_STATUSES.includes(o.status)).length;
  const pendingCompliance = mockComplianceCases.filter((c) => c.status === "PENDING" || c.status === "REQUIRES_ACTION").length;
  const mismatches = mockReconciliationRecords.filter((r) => r.status === "MISMATCH" || r.status === "UNDER_REVIEW").length;

  return (
    <PageContainer>
      <PageHeader title="Admin Dashboard" description="Platform-wide overview of assets, investors, and operations." />

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardContent className="pt-6">
            <FinancialMetric label="Total AUM" value={<Money value={totalAum} compact />} />
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <FinancialMetric label="Investors" value={mockInvestors.length} />
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <FinancialMetric label="Open Orders" value={openOrders} />
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <FinancialMetric label="Compliance Cases" value={pendingCompliance} />
          </CardContent>
        </Card>
      </div>

      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Link href="/admin/investors">
          <Card className="card-interactive">
            <CardContent className="flex items-center gap-3 pt-6">
              <Users className="size-5 text-muted-foreground" />
              <span className="text-sm font-medium">Manage Investors</span>
            </CardContent>
          </Card>
        </Link>
        <Link href="/admin/assets">
          <Card className="card-interactive">
            <CardContent className="flex items-center gap-3 pt-6">
              <Building2 className="size-5 text-muted-foreground" />
              <span className="text-sm font-medium">Manage Assets</span>
            </CardContent>
          </Card>
        </Link>
        <Link href="/admin/compliance">
          <Card className="card-interactive">
            <CardContent className="flex items-center gap-3 pt-6">
              <ScanEye className="size-5 text-muted-foreground" />
              <span className="text-sm font-medium">Compliance Queue</span>
            </CardContent>
          </Card>
        </Link>
        <Link href="/admin/reconciliation">
          <Card className="card-interactive">
            <CardContent className="flex items-center gap-3 pt-6">
              <AlertTriangle className="size-5 text-muted-foreground" />
              <span className="text-sm font-medium">{mismatches} Reconciliation Alerts</span>
            </CardContent>
          </Card>
        </Link>
      </div>

      <div className="mt-6">
        <Card>
          <CardContent className="flex items-center gap-3 pt-6">
            <ListOrdered className="size-5 text-muted-foreground" />
            <p className="text-sm text-muted-foreground">
              {mockOrders.length} total orders across {mockAssets.length} listed assets.
            </p>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  );
}
