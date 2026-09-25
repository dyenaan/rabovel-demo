import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { MarketsTable } from "@/features/markets/components/markets-table";

export const metadata: Metadata = { title: "Markets" };

export default function MarketsPage() {
  return (
    <PageContainer>
      <PageHeader
        title="Markets"
        description="Live pricing and 24-hour activity across all secondary markets."
      />
      <MarketsTable />
    </PageContainer>
  );
}
