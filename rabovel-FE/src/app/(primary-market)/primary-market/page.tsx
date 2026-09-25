import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { PrimaryMarketDashboard } from "@/features/primary-market/components/primary-market-dashboard";

export const metadata: Metadata = { title: "Primary Market" };

export default function PrimaryMarketPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Primary Market"
        description="Subscribe to new issuances and manage redemptions at NAV."
      />
      <PrimaryMarketDashboard />
    </PageContainer>
  );
}
