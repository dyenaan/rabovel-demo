import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { ComplianceSummaryCard } from "@/features/dashboard/components/compliance-summary-card";
import { AssetAllocationChart } from "@/features/portfolio/components/asset-allocation-chart";
import { PortfolioActivity } from "@/features/portfolio/components/portfolio-activity";
import { PortfolioPerformanceChart } from "@/features/portfolio/components/portfolio-performance-chart";
import { PortfolioSummaryCards } from "@/features/portfolio/components/portfolio-summary-cards";

export const metadata: Metadata = { title: "Dashboard" };

export default function DashboardPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Dashboard"
        description="Your portfolio, market activity, and compliance status at a glance."
      />

      <div className="space-y-6">
        <PortfolioSummaryCards />

        <PortfolioPerformanceChart />

        <div className="grid gap-6 lg:grid-cols-3">
          <AssetAllocationChart />
          <ComplianceSummaryCard />
          <PortfolioActivity />
        </div>
      </div>
    </PageContainer>
  );
}
